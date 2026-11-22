use aether_document::{Document, NoOpDocument};
use blitz_shell::{BlitzShellEvent, BlitzShellProxy};
use winit::window::WindowId;

use crate::{AetherNativeEvent, AetherNativeWindowRenderer, Config, RendererOptions, WindowConfig};
use aether_core::{Element, VirtualDom, consume_context, use_hook};
use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::{Rc, Weak},
};

type PendingWindows = RefCell<VecDeque<WindowConfig<AetherNativeWindowRenderer>>>;
pub(crate) type NativeWindowQueue = Rc<PendingWindows>;

/// UI-thread handle for opening and managing native windows.
#[derive(Clone)]
pub struct NativeAppHandle {
    pub(crate) proxy: BlitzShellProxy,
    pub(crate) queue: Weak<PendingWindows>,
}

impl NativeAppHandle {
    fn send(&self, event: AetherNativeEvent) {
        self.proxy
            .send_event(BlitzShellEvent::embedder_event(event));
    }

    /// Queue a new window from the UI thread.
    pub fn open_window(&self, vdom: VirtualDom, config: Config) {
        if let Some(queue) = self.queue.upgrade() {
            let window = crate::build_window(vdom, config, &self.proxy, RendererOptions::default());
            queue.borrow_mut().push_back(window);
            self.send(AetherNativeEvent::OpenAuxiliaryWindow);
        }
    }

    pub fn open_component_window(&self, app: fn() -> Element, config: Config) {
        self.open_window(VirtualDom::new(app), config);
    }

    pub fn request_close(&self, window_id: WindowId) {
        self.send(AetherNativeEvent::CloseWindow(window_id));
    }

    pub fn set_window_visible(&self, window: WindowId, visible: bool) {
        self.send(AetherNativeEvent::SetWindowVisible { window, visible });
    }
}

pub fn use_native_app() -> NativeAppHandle {
    use_hook(consume_context::<NativeAppHandle>)
}

pub struct AetherNativeDocument {
    pub(crate) proxy: BlitzShellProxy,
    pub(crate) window: WindowId,
}

impl AetherNativeDocument {
    pub(crate) fn new(proxy: BlitzShellProxy, window: WindowId) -> Self {
        Self { proxy, window }
    }
}

impl Document for AetherNativeDocument {
    fn eval(&self, _js: String) -> aether_document::Eval {
        NoOpDocument.eval(_js)
    }

    fn create_head_element(
        &self,
        name: &str,
        attributes: &[(&str, String)],
        contents: Option<String>,
    ) {
        let window = self.window;
        self.proxy.send_event(BlitzShellEvent::embedder_event(
            AetherNativeEvent::CreateHeadElement {
                name: name.to_string(),
                attributes: attributes
                    .iter()
                    .map(|(name, value)| (name.to_string(), value.clone()))
                    .collect(),
                contents,
                window,
            },
        ));
    }

    fn set_title(&self, title: String) {
        self.create_head_element("title", &[], Some(title));
    }

    fn create_meta(&self, props: aether_document::MetaProps) {
        let attributes = props.attributes();
        self.create_head_element("meta", &attributes, None);
    }

    fn create_script(&self, props: aether_document::ScriptProps) {
        let attributes = props.attributes();
        self.create_head_element("script", &attributes, props.script_contents().ok());
    }

    fn create_style(&self, props: aether_document::StyleProps) {
        let attributes = props.attributes();
        self.create_head_element("style", &attributes, props.style_contents().ok());
    }

    fn create_link(&self, props: aether_document::LinkProps) {
        let attributes = props.attributes();
        self.create_head_element("link", &attributes, None);
    }

    fn create_head_component(&self) -> bool {
        true
    }
}
