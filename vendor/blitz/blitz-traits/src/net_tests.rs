// Copyright (c) 2026 Timothy Charles Holborn. Aether repository licensing applies.
use super::*;
use std::{
    future::Future,
    pin::Pin,
    sync::{Mutex, atomic::AtomicUsize},
    task::{Context, Wake, Waker},
};

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
fn abort_wakes_every_waiter_and_releases_registration() {
    let controller = AbortController::default();
    let signal = controller.signal.clone();
    let counter = Arc::new(Counter(AtomicUsize::new(0)));
    let waker = Waker::from(counter.clone());
    let mut cx = Context::from_waker(&waker);
    let mut first = signal.cancelled();
    let mut second = signal.cancelled();
    assert!(Pin::new(&mut first).poll(&mut cx).is_pending());
    assert!(Pin::new(&mut second).poll(&mut cx).is_pending());
    controller.abort();
    assert_eq!(counter.0.load(Ordering::SeqCst), 2);
    assert!(Pin::new(&mut first).poll(&mut cx).is_ready());
    assert!(Pin::new(&mut second).poll(&mut cx).is_ready());
    assert!(signal.0.waiters.lock().unwrap().is_empty());
}

#[test]
fn completed_waiter_unregisters_and_slot_reuse_does_not_cancel_another() {
    let controller = AbortController::default();
    let signal = controller.signal.clone();
    let counter = Arc::new(Counter(AtomicUsize::new(0)));
    let waker = Waker::from(counter.clone());
    let mut cx = Context::from_waker(&waker);
    let mut retired = signal.cancelled();
    let mut retained = signal.cancelled();
    assert!(Pin::new(&mut retired).poll(&mut cx).is_pending());
    assert!(Pin::new(&mut retained).poll(&mut cx).is_pending());
    drop(retired);
    let mut replacement = signal.cancelled();
    assert!(Pin::new(&mut replacement).poll(&mut cx).is_pending());
    drop(retained);
    controller.abort();
    assert_eq!(counter.0.load(Ordering::SeqCst), 1);
    assert!(Pin::new(&mut replacement).poll(&mut cx).is_ready());
}

struct ReentrantDrop(AbortSignal);
impl Wake for ReentrantDrop {
    fn wake(self: Arc<Self>) {}
}
impl Drop for ReentrantDrop {
    fn drop(&mut self) {
        assert!(
            self.0.0.waiters.try_lock().is_ok(),
            "waker dropped under registry lock"
        );
    }
}

#[test]
fn replacing_and_removing_wakers_drops_outside_the_lock() {
    let signal = AbortSignal::default();
    let mut future = signal.cancelled();
    let old = Waker::from(Arc::new(ReentrantDrop(signal.clone())));
    assert!(
        Pin::new(&mut future)
            .poll(&mut Context::from_waker(&old))
            .is_pending()
    );
    drop(old);
    let new = Waker::from(Arc::new(ReentrantDrop(signal.clone())));
    assert!(
        Pin::new(&mut future)
            .poll(&mut Context::from_waker(&new))
            .is_pending()
    );
    drop(new);
    drop(future);
}

#[derive(Default)]
struct Recorder(Mutex<Vec<Result<Vec<u8>, NetError>>>);
struct Handler(Arc<Recorder>);
impl NetHandler for Handler {
    fn bytes(self: Box<Self>, _: String, bytes: Bytes) {
        self.0.0.lock().unwrap().push(Ok(bytes.to_vec()));
    }
    fn failed(self: Box<Self>, _: String, error: NetError) {
        self.0.0.lock().unwrap().push(Err(error));
    }
}
#[test]
fn terminal_completion_is_exactly_once_on_success_error_or_drop() {
    let recorder = Arc::new(Recorder::default());
    let make = || {
        CompletionHandler::boxed(
            "https://example.invalid/asset".into(),
            Box::new(Handler(recorder.clone())),
        )
    };
    make().bytes(
        "https://example.invalid/asset".into(),
        Bytes::from_static(b"ok"),
    );
    make().failed(
        "https://example.invalid/asset".into(),
        NetError::AccessDenied,
    );
    drop(make());
    assert_eq!(
        *recorder.0.lock().unwrap(),
        vec![
            Ok(b"ok".to_vec()),
            Err(NetError::AccessDenied),
            Err(NetError::Cancelled)
        ]
    );
}

#[test]
fn late_transport_success_does_not_publish_cancelled_bytes() {
    let controller = AbortController::default();
    let recorder = Arc::new(Recorder::default());
    let handler = CompletionHandler::with_signal(
        "data:late".into(),
        Some(controller.signal.clone()),
        Box::new(Handler(recorder.clone())),
    );
    controller.abort();
    handler.bytes("data:late".into(), Bytes::from_static(b"private"));
    assert_eq!(*recorder.0.lock().unwrap(), vec![Err(NetError::Cancelled)]);
}
