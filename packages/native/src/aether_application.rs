use aether_core::{ScopeId, provide_context};
use aether_history::{History, MemoryHistory};
use blitz_dom::BaseDocument;
use blitz_shell::{BlitzApplication, BlitzShellProxy, View};
use std::sync::Arc;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

#[cfg(target_os = "macos")]
use winit::platform::macos::ApplicationHandlerExtMacOS;

use crate::AetherNativeWindowRenderer;
use crate::event_handlers::WindowEventHandlers;
use crate::{
    AetherDocument, BlitzShellEvent, NativeAppHandle, WindowClosePolicy, WindowConfig,
    contexts::{AetherNativeDocument, NativeWindowQueue},
};

/// Events carried by the Blitz shell proxy.
pub enum AetherNativeEvent {
    /// Wake the UI thread for a configuration queued by the native app handle.
    /// The configuration stays on the UI thread because it owns the VirtualDOM.
    #[doc(hidden)]
    OpenAuxiliaryWindow,
    CloseWindow(WindowId),
    SetWindowVisible {
        window: WindowId,
        visible: bool,
    },

    /// A hotreload event, basically telling us to update our templates.
    #[cfg(all(feature = "hot-reload", debug_assertions))]
    DevserverEvent(aether_devtools::DevserverMsg),

    /// Create a new head element from the Link and Title elements
    ///
    /// todo(jon): these should probabkly be synchronous somehow
    CreateHeadElement {
        window: WindowId,
        name: String,
        attributes: Vec<(String, String)>,
        contents: Option<String>,
    },

    /// Dispatch a synthetic UI event (PointerMove, PointerDown, PointerUp, KeyDown, Ime, etc.)
    /// into a live window's document and trigger a poll + redraw.
    DispatchUiEvent {
        window: WindowId,
        event: blitz_traits::events::UiEvent,
    },

    /// Click a specific DOM node by `NodeId`, scrolling it into view or synthesizing a click
    /// if off-screen, and trigger a poll + redraw.
    ClickNode {
        window: WindowId,
        node_id: blitz_traits::node_id::NodeId,
    },
}

pub struct AetherNativeApplication {
    surfaces_available: bool,
    inner: BlitzApplication<AetherNativeWindowRenderer>,
    event_handlers: Rc<WindowEventHandlers>,
    auxiliary_windows: NativeWindowQueue,
    close_policies: HashMap<WindowId, WindowClosePolicy>,
}

impl AetherNativeApplication {
    pub fn new(
        proxy: BlitzShellProxy,
        event_queue: std::sync::mpsc::Receiver<BlitzShellEvent>,
        config: WindowConfig<AetherNativeWindowRenderer>,
    ) -> Self {
        let mut inner = BlitzApplication::new(proxy, event_queue);
        inner.add_window(config);
        Self {
            surfaces_available: false,
            inner,
            event_handlers: Rc::new(WindowEventHandlers::default()),
            auxiliary_windows: Rc::new(RefCell::new(Default::default())),
            close_policies: HashMap::new(),
        }
    }

    pub fn add_window(&mut self, window_config: WindowConfig<AetherNativeWindowRenderer>) {
        self.inner.add_window(window_config);
        self.inner.proxy.wake_up();
    }

    // All initial and later windows go through this preparation path. Never let
    // the underlying generic shell consume pending Aether windows before their
    // document, history, renderer, shell and event-handler contexts exist.
    fn prepare_pending_windows(&mut self, event_loop: &dyn ActiveEventLoop) {
        if !self.surfaces_available {
            return;
        }
        for config in std::mem::take(&mut self.inner.pending_windows) {
            let mut window = View::init(config, event_loop, &self.inner.proxy);
            let winit_window = Arc::clone(&window.window);
            let renderer = window.renderer.clone();
            let window_id = window.window_id();
            let doc = window.downcast_doc_mut::<AetherDocument>();

            let policy = doc.vdom.in_scope(ScopeId::ROOT, || {
                aether_core::try_consume_context::<WindowClosePolicy>().unwrap_or_default()
            });
            self.close_policies.insert(window_id, policy);
            let handle = NativeAppHandle {
                proxy: self.inner.proxy.clone(),
                queue: Rc::downgrade(&self.auxiliary_windows),
            };
            let base_doc: Rc<RefCell<BaseDocument>> = Rc::clone(&doc.inner);
            doc.vdom.in_scope(ScopeId::ROOT, || {
                provide_context(handle);
                let shared: Rc<dyn aether_document::Document> = Rc::new(AetherNativeDocument::new(
                    self.inner.proxy.clone(),
                    window_id,
                ));
                provide_context(shared);
                provide_context(self.event_handlers.clone());
                provide_context(base_doc);
            });

            // Add shell provider
            let shell_provider = doc.inner.borrow().shell_provider.clone();
            doc.vdom
                .in_scope(ScopeId::ROOT, move || provide_context(shell_provider));

            // Add history
            let history_provider: Rc<dyn History> = Rc::new(MemoryHistory::default());
            doc.vdom
                .in_scope(ScopeId::ROOT, move || provide_context(history_provider));

            // Add renderer
            doc.vdom
                .in_scope(ScopeId::ROOT, move || provide_context(renderer));

            // Add winit window
            doc.vdom
                .in_scope(ScopeId::ROOT, move || provide_context(winit_window));

            // Queue rebuild
            doc.initial_build();

            // Newly created windows need their own resume after contexts/rebuild.
            window.resume();
            window.request_redraw();

            self.inner.windows.insert(window_id, window);
        }
    }

    fn set_visible(&mut self, id: WindowId, visible: bool) {
        if let Some(window) = self.inner.windows.get_mut(&id) {
            if !visible {
                window
                    .downcast_doc_mut::<AetherDocument>()
                    .inner
                    .borrow_mut()
                    .set_window_focused(false);
            }
            window.is_visible = visible;
            window.window.set_visible(visible);
            if visible {
                window.request_redraw();
            }
        }
    }

    fn close_window(&mut self, id: WindowId) -> bool {
        if self.close_policies.get(&id) == Some(&WindowClosePolicy::HideOnClose) {
            self.set_visible(id, false);
            return false;
        }
        self.close_policies.remove(&id);
        if let Some(mut window) = self.inner.windows.remove(&id) {
            window.suspend();
            return true;
        }
        false
    }

    fn exit_if_last_window_closed(&self, event_loop: &dyn ActiveEventLoop, closed: bool) {
        if closed
            && self.inner.windows.is_empty()
            && self.inner.pending_windows.is_empty()
            && self.auxiliary_windows.borrow().is_empty()
        {
            event_loop.exit();
        }
    }

    fn handle_proxy_event(&mut self, event_loop: &dyn ActiveEventLoop, event: &AetherNativeEvent) {
        match event {
            #[cfg(all(feature = "hot-reload", debug_assertions))]
            AetherNativeEvent::DevserverEvent(event) => match event {
                aether_devtools::DevserverMsg::HotReload(hotreload_message) => {
                    for window in self.inner.windows.values_mut() {
                        let doc = window.downcast_doc_mut::<AetherDocument>();

                        // Apply changes to vdom
                        aether_devtools::apply_changes(&doc.vdom, hotreload_message);

                        // Reload changed assets
                        for asset_path in &hotreload_message.assets {
                            if let Some(url) = asset_path.to_str() {
                                doc.inner.borrow_mut().reload_resource_by_href(url);
                            }
                        }

                        window.poll();
                    }
                }
                aether_devtools::DevserverMsg::Shutdown => event_loop.exit(),
                aether_devtools::DevserverMsg::FullReloadStart => {}
                aether_devtools::DevserverMsg::FullReloadFailed => {}
                aether_devtools::DevserverMsg::FullReloadCommand => {}
                _ => {}
            },

            AetherNativeEvent::CreateHeadElement {
                name,
                attributes,
                contents,
                window,
            } => {
                if let Some(window) = self.inner.windows.get_mut(window) {
                    let doc = window.downcast_doc_mut::<AetherDocument>();
                    doc.create_head_element(name, attributes, contents);
                    window.poll();
                }
            }

            AetherNativeEvent::OpenAuxiliaryWindow => {
                let config = self.auxiliary_windows.borrow_mut().pop_front();
                if let Some(config) = config {
                    self.inner.add_window(config);
                    self.prepare_pending_windows(event_loop);
                }
            }
            AetherNativeEvent::CloseWindow(id) => {
                let closed = self.close_window(*id);
                self.exit_if_last_window_closed(event_loop, closed);
            }
            AetherNativeEvent::SetWindowVisible { window, visible } => {
                self.set_visible(*window, *visible);
            }
            AetherNativeEvent::DispatchUiEvent { window, event } => {
                if let Some(window) = self.inner.windows.get_mut(window) {
                    window.doc.handle_ui_event(event.clone());
                    window.poll();
                    window.request_redraw();
                }
            }
            AetherNativeEvent::ClickNode { window, node_id } => {
                if let Some(window) = self.inner.windows.get_mut(window) {
                    let doc = window.downcast_doc_mut::<AetherDocument>();
                    doc.click_node(*node_id);
                    window.poll();
                    window.request_redraw();
                }
            }
        }
    }
}

impl ApplicationHandler for AetherNativeApplication {
    #[cfg(target_os = "macos")]
    fn macos_handler(&mut self) -> Option<&mut dyn ApplicationHandlerExtMacOS> {
        self.inner.macos_handler()
    }

    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.resumed(event_loop);
    }

    fn suspended(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.suspended(event_loop);
    }

    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.surfaces_available = false;
        self.inner.destroy_surfaces(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.prepare_pending_windows(event_loop);
        self.inner.about_to_wait(event_loop);
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.surfaces_available = true;
        // Let Blitz resume existing windows, but retain Aether-specific ownership
        // of new-window context injection and the initial VirtualDOM rebuild.
        let pending = std::mem::take(&mut self.inner.pending_windows);
        self.inner.can_create_surfaces(event_loop);
        self.inner.pending_windows.extend(pending);
        self.prepare_pending_windows(event_loop);
    }

    fn new_events(&mut self, event_loop: &dyn ActiveEventLoop, cause: StartCause) {
        self.inner.new_events(event_loop, cause);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        self.event_handlers
            .apply_event(window_id, &event, event_loop);
        self.prepare_pending_windows(event_loop);
        if matches!(event, WindowEvent::CloseRequested) {
            let closed = self.close_window(window_id);
            self.exit_if_last_window_closed(event_loop, closed);
        } else {
            if let WindowEvent::Focused(focused) = &event {
                if let Some(window) = self.inner.windows.get_mut(&window_id) {
                    window
                        .downcast_doc_mut::<AetherDocument>()
                        .inner
                        .borrow_mut()
                        .set_window_focused(*focused);
                }
            }
            let poll_after_redraw = matches!(event, WindowEvent::RedrawRequested);
            self.inner.window_event(event_loop, window_id, event);
            if poll_after_redraw {
                if let Some(window) = self.inner.windows.get_mut(&window_id) {
                    window.poll();
                }
            }
        }
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.prepare_pending_windows(event_loop);
        while let Ok(event) = self.inner.event_queue.try_recv() {
            match event {
                BlitzShellEvent::Embedder(event) => {
                    if let Some(event) = event.downcast_ref::<AetherNativeEvent>() {
                        self.handle_proxy_event(event_loop, event);
                    }
                }
                BlitzShellEvent::CloseWindow { window_id } => {
                    let closed = self.close_window(window_id);
                    self.exit_if_last_window_closed(event_loop, closed);
                }
                event => self.inner.handle_blitz_shell_event(event_loop, event),
            }
        }
    }
}
