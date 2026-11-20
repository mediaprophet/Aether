// Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
use super::*;
use blitz_traits::net::AbortController;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Wake, Waker};

struct Counter(AtomicUsize);
impl Wake for Counter {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
#[test]
fn abort_wakes_a_transport_that_never_completes() {
    let controller = AbortController::default();
    let counter = Arc::new(Counter(AtomicUsize::new(0)));
    let waker = Waker::from(counter.clone());
    let mut cx = Context::from_waker(&waker);
    let mut request = AbortFetch::new(
        controller.signal.clone(),
        Box::pin(std::future::pending::<Result<(), ProviderError>>()),
    );
    assert!(Pin::new(&mut request).poll(&mut cx).is_pending());
    controller.abort();
    assert_eq!(counter.0.load(Ordering::SeqCst), 1);
    assert!(matches!(
        Pin::new(&mut request).poll(&mut cx),
        Poll::Ready(Err(ProviderError::Abort))
    ));
}

#[test]
fn public_async_and_callback_requests_honor_preexisting_cancellation() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let provider = Provider::new(None);
        let controller = AbortController::default();
        let signal = controller.signal.clone();
        controller.abort();
        let request = || {
            Request::get("data:text/plain,never-deliver".parse().unwrap()).signal(signal.clone())
        };
        assert!(matches!(
            provider.fetch_async(request()).await,
            Err(ProviderError::Abort)
        ));
        let (tx, rx) = tokio::sync::oneshot::channel();
        provider.fetch_with_callback(
            request(),
            Box::new(move |result| {
                tx.send(result).ok();
            }),
        );
        assert!(matches!(rx.await.unwrap(), Err(ProviderError::Abort)));
    });
}
