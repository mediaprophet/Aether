// Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
use super::*;
use blitz_traits::net::{AbortController, Bytes};
use std::sync::Mutex;

type Outcomes = Arc<Mutex<Vec<Result<Vec<u8>, NetError>>>>;
struct Handler(Outcomes);
impl NetHandler for Handler {
    fn bytes(self: Box<Self>, _: String, bytes: Bytes) {
        self.0.lock().unwrap().push(Ok(bytes.to_vec()));
    }
    fn failed(self: Box<Self>, _: String, error: NetError) {
        self.0.lock().unwrap().push(Err(error));
    }
}
fn fetch(
    url: &str,
    abort: bool,
    inner: Option<Arc<dyn NetProvider>>,
) -> Vec<Result<Vec<u8>, NetError>> {
    let results = Outcomes::default();
    let provider = AetherNativeNetProvider {
        asset_access: AssetAccess::Packaged,
        max_bytes: aether_asset_resolver::native::DEFAULT_ASSET_MAX_BYTES,
        inner_net_provider: inner,
    };
    let control = AbortController::default();
    let request = Request::get(url.parse().unwrap()).signal(control.signal.clone());
    if abort {
        control.abort();
    }
    provider.fetch(0, request, Box::new(Handler(results.clone())));
    Arc::try_unwrap(results).ok().unwrap().into_inner().unwrap()
}
#[test]
fn failed_assets_complete_with_status_instead_of_bytes() {
    assert_eq!(
        fetch("aether://app/missing-resource-762674.txt", false, None),
        vec![Err(NetError::HttpStatus(404))]
    );
}
#[test]
fn denied_unavailable_and_cancelled_requests_complete_once() {
    assert_eq!(
        fetch("file:///private.txt", false, None),
        vec![Err(NetError::AccessDenied)]
    );
    assert_eq!(
        fetch("https://example.invalid/", false, None),
        vec![Err(NetError::Unavailable)]
    );
    assert_eq!(
        fetch("aether://app/anything", true, None),
        vec![Err(NetError::Cancelled)]
    );
}
struct DroppingTransport;
impl NetProvider for DroppingTransport {
    fn fetch(&self, _: usize, _: Request, _: Box<dyn NetHandler>) {}
}
#[test]
fn legacy_transport_drop_finishes_the_pending_request() {
    assert_eq!(
        fetch("data:invalid", false, Some(Arc::new(DroppingTransport))),
        vec![Err(NetError::Cancelled)]
    );
}

#[test]
fn explicit_host_byte_limit_is_applied_to_trusted_asset_requests() {
    let path = std::env::temp_dir().join(format!(
        "aether-asset-limit-{}-{}.txt",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        file.write_all(b"asset").unwrap();
    }
    let mut url: blitz_traits::net::Url = "aether://app/".parse().unwrap();
    url.set_path(&path.to_string_lossy().replace('\\', "/"));
    for (limit, accepted) in [(4, false), (5, true)] {
        let results = Outcomes::default();
        let provider = AetherNativeNetProvider {
            asset_access: AssetAccess::TrustedApplicationFiles,
            max_bytes: aether_asset_resolver::native::DEFAULT_ASSET_MAX_BYTES,
            inner_net_provider: None,
        }
        .with_asset_max_bytes(limit);
        provider.fetch(
            0,
            Request::get(url.clone()),
            Box::new(Handler(results.clone())),
        );
        let outcomes = results.lock().unwrap();
        assert_eq!(outcomes.len(), 1);
        if accepted {
            assert_eq!(outcomes[0], Ok(b"asset".to_vec()));
        } else {
            assert!(matches!(outcomes[0], Err(NetError::Transport(_))));
        }
    }
    std::fs::remove_file(path).unwrap();
}
