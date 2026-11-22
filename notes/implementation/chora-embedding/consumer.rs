// Compile-only external Chora consumer. This grants no host service authority.
use chora::{
    ChoraCallerContext, ChoraMessageError, ChoraMessageId, EmbedderMessageRequest,
    WebView, WebViewDelegate,
};

struct Host;
impl WebViewDelegate for Host {
    fn notify_embedder_message(&self, _view: WebView, request: EmbedderMessageRequest) {
        let caller: &ChoraCallerContext = request.caller();
        let _route = (caller.engine_instance(), caller.webview_id(), caller.pipeline_id(),
            caller.navigation_epoch(), caller.origin(), caller.is_private_browsing());
        let _request_id = request.id();
        // A real host first maps the engine to its profile and authorizes the operation.
        request.respond(Err(ChoraMessageError::Denied));
    }
    fn notify_embedder_message_cancelled(&self, _view: WebView, _id: ChoraMessageId, _reason: ChoraMessageError) {}
}
fn main() { let _delegate: &dyn WebViewDelegate = &Host; }
