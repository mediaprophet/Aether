use blitz_shell::{BlitzShellEvent, BlitzShellProxy};
use aether_document::{Document, NoOpDocument};
use winit::window::WindowId;

use crate::AetherNativeEvent;

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
