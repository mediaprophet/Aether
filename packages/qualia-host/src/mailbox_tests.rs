// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
use crate::{UiDelivery, ui_channel};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
};

#[derive(Default)]
struct WakeCount(AtomicUsize);
impl Wake for WakeCount {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn async_ui_receiver_wakes_and_drains_results_before_producer_eof() {
    let (sender, mut receiver) = ui_channel(1, || {}).unwrap();
    let count = Arc::new(WakeCount::default());
    let waker = Waker::from(count.clone());
    let mut context = Context::from_waker(&waker);
    assert!(receiver.poll_next(&mut context).is_pending());
    sender
        .publish(
            sender.begin().unwrap(),
            UiDelivery {
                value: vibe::Value::I64(9),
                receipts: vec![],
            },
        )
        .unwrap();
    assert_eq!(count.0.load(Ordering::SeqCst), 1);
    drop(sender);
    assert!(
        matches!(receiver.poll_next(&mut context), Poll::Ready(Some(delivery))
        if delivery.value.as_i64() == Some(9))
    );
    assert!(matches!(
        receiver.poll_next(&mut context),
        Poll::Ready(None)
    ));
}

#[test]
fn async_ui_receiver_replaces_waiter_and_wakes_on_empty_producer_shutdown() {
    let (sender, mut receiver) = ui_channel(1, || {}).unwrap();
    let first = Arc::new(WakeCount::default());
    let second = Arc::new(WakeCount::default());
    let first_waker = Waker::from(first.clone());
    let second_waker = Waker::from(second.clone());
    assert!(
        receiver
            .poll_next(&mut Context::from_waker(&first_waker))
            .is_pending()
    );
    assert!(
        receiver
            .poll_next(&mut Context::from_waker(&second_waker))
            .is_pending()
    );
    drop(sender);
    assert_eq!(first.0.load(Ordering::SeqCst), 0);
    assert_eq!(second.0.load(Ordering::SeqCst), 1);
    assert!(matches!(
        receiver.poll_next(&mut Context::from_waker(&second_waker)),
        Poll::Ready(None)
    ));
}

#[test]
fn native_receiver_teardown_releases_retained_task_waker() {
    let (sender, mut receiver) = ui_channel(1, || {}).unwrap();
    let owner = Arc::new(WakeCount::default());
    let weak_owner = Arc::downgrade(&owner);
    let waker = Waker::from(owner.clone());
    assert!(
        receiver
            .poll_next(&mut Context::from_waker(&waker))
            .is_pending()
    );
    drop(waker);
    drop(owner);
    assert!(weak_owner.upgrade().is_some());
    drop(receiver);
    assert!(weak_owner.upgrade().is_none());
    assert!(sender.begin().is_err());
}

// A scheduler task may own the runtime/sender. Releasing its last stored waker
// then drops UiSender, which must be able to reacquire the mailbox mutex.
struct SenderOwner(crate::UiSender);
impl Wake for SenderOwner {
    fn wake(self: Arc<Self>) {
        let _ = self.0.begin();
    }
}

fn reentrant_waiter_release_completes(replace: bool) {
    let (done, completed) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let (sender, mut receiver) = ui_channel(1, || {}).unwrap();
        let waker = Waker::from(Arc::new(SenderOwner(sender)));
        assert!(
            receiver
                .poll_next(&mut Context::from_waker(&waker))
                .is_pending()
        );
        drop(waker);
        if replace {
            let replacement = Waker::from(Arc::new(WakeCount::default()));
            assert!(
                receiver
                    .poll_next(&mut Context::from_waker(&replacement))
                    .is_pending()
            );
            assert!(matches!(
                receiver.poll_next(&mut Context::from_waker(&replacement)),
                Poll::Ready(None)
            ));
        }
        drop(receiver);
        done.send(()).unwrap();
    });
    completed
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("releasing the retained waker must not deadlock on sender teardown");
}

#[test]
fn native_receiver_teardown_can_drop_a_sender_owned_by_its_waiter() {
    reentrant_waiter_release_completes(false);
}

#[test]
fn async_waiter_replacement_can_drop_a_sender_owned_by_the_previous_waiter() {
    reentrant_waiter_release_completes(true);
}
