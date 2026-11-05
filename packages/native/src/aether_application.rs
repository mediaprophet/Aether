use blitz_shell::{BlitzApplication, BlitzShellProxy, View};
use aether_core::{ScopeId, provide_context};
use aether_history::{History, MemoryHistory};
use std::rc::Rc;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

#[cfg(target_os = "macos")]
use winit::platform::macos::ApplicationHandlerExtMacOS;

use crate::AetherNativeWindowRenderer;
use crate::event_handlers::WindowEventHandlers;
use crate::{BlitzShellEvent, AetherDocument, WindowConfig, contexts::AetherNativeDocument};

/// Aether-native specific event type
pub enum AetherNativeEvent {
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
}

pub struct AetherNativeApplication {
    pending_window: Option<WindowConfig<AetherNativeWindowRenderer>>,
    inner: BlitzApplication<AetherNativeWindowRenderer>,
    event_handlers: Rc<WindowEventHandlers>,
}

impl AetherNativeApplication {
    pub fn new(
        proxy: BlitzShellProxy,
        event_queue: std::sync::mpsc::Receiver<BlitzShellEvent>,
        config: WindowConfig<AetherNativeWindowRenderer>,
    ) -> Self {
        Self {
            pending_window: Some(config),
            inner: BlitzApplication::new(proxy, event_queue),
            event_handlers: Rc::new(WindowEventHandlers::default()),
        }
    }

    pub fn add_window(&mut self, window_config: WindowConfig<AetherNativeWindowRenderer>) {
        self.inner.add_window(window_config);
    }

    fn handle_aether_native_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: &AetherNativeEvent,
    ) {
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

            // Suppress unused variable warning
            #[cfg(not(all(feature = "hot-reload", debug_assertions)))]
            #[allow(unreachable_patterns)]
            _ => {
                let _ = event_loop;
                let _ = event;
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
        self.inner.destroy_surfaces(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.inner.about_to_wait(event_loop);
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        #[cfg(feature = "tracing")]
        tracing::debug!("Injecting document provider into all windows");

        if let Some(config) = self.pending_window.take() {
            let mut window = View::init(config, event_loop, &self.inner.proxy);
            let winit_window = Arc::clone(&window.window);
            let renderer = window.renderer.clone();
            let window_id = window.window_id();
            let doc = window.downcast_doc_mut::<AetherDocument>();

            doc.vdom.in_scope(ScopeId::ROOT, || {
                let shared: Rc<dyn aether_document::Document> = Rc::new(AetherNativeDocument::new(
                    self.inner.proxy.clone(),
                    window_id,
                ));
                provide_context(shared);
                provide_context(self.event_handlers.clone());
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

            // And then request redraw
            window.request_redraw();

            // todo(jon): we should actually mess with the pending windows instead of passing along the contexts
            self.inner.windows.insert(window_id, window);
        }

        self.inner.can_create_surfaces(event_loop);
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
        self.inner.window_event(event_loop, window_id, event);
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        while let Ok(event) = self.inner.event_queue.try_recv() {
            match event {
                BlitzShellEvent::Embedder(event) => {
                    if let Some(event) = event.downcast_ref::<AetherNativeEvent>() {
                        self.handle_aether_native_event(event_loop, event);
                    }
                }
                event => self.inner.handle_blitz_shell_event(event_loop, event),
            }
        }
    }
}
