// Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
use super::*;
use crate::{BaseDocument, DocumentConfig};

#[test]
fn failed_resource_releases_critical_render_wait() {
    let mut document = BaseDocument::new(DocumentConfig::default());
    let handler = ResourceHandler::new(
        document.tx.clone(),
        0,
        None,
        document.shell_provider.clone(),
        DocumentSrcHandler,
    );
    document
        .pending_critical_resources
        .insert(handler.request_id());
    assert!(document.has_pending_critical_resources());
    Box::new(handler).failed(
        "https://example.invalid/asset".into(),
        NetError::AccessDenied,
    );
    document.handle_messages();
    assert!(!document.has_pending_critical_resources());
}

struct FailedImportProvider {
    malformed: bool,
}
impl NetProvider for FailedImportProvider {
    fn fetch(&self, _: usize, request: Request, handler: Box<dyn NetHandler>) {
        if self.malformed {
            handler.bytes(request.url.to_string(), Bytes::from_static(&[0xff]));
        } else {
            handler.failed(request.url.to_string(), NetError::HttpStatus(404));
        }
    }
}

#[test]
fn failed_or_malformed_import_reaches_a_terminal_cssom_state() {
    for malformed in [false, true] {
        let mut document = BaseDocument::new(DocumentConfig::default());
        let loader = StylesheetLoader {
            tx: document.tx.clone(),
            doc_id: 0,
            net_provider: Arc::new(FailedImportProvider { malformed }),
            shell_provider: document.shell_provider.clone(),
            abort_signal: None,
            import_depth: 0,
        };
        let sheet = Stylesheet::from_str(
            "@import url('missing.css');",
            Url::parse("https://example.invalid/main.css")
                .unwrap()
                .into(),
            Origin::Author,
            ServoArc::new(document.guard.wrap(MediaList::empty())),
            document.guard.clone(),
            Some(&loader),
            None,
            QuirksMode::NoQuirks,
            AllowImportRules::Yes,
        );
        let rule = {
            let guard = document.guard.read();
            let CssRule::Import(rule) = &sheet.contents(&guard).rules(&guard)[0] else {
                panic!("missing import");
            };
            assert!(matches!(
                rule.read_with(&guard).stylesheet,
                ImportSheet::Pending
            ));
            rule.clone()
        };
        document.handle_messages();
        assert!(matches!(
            rule.read_with(&document.guard.read()).stylesheet,
            ImportSheet::Refused
        ));
    }
}
