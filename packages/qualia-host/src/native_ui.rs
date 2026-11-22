// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Component-owned delivery of canonical values on Aether's UI scheduler.

pub use crate::native_graph_ui::use_qualia_graph;
use crate::{UiDelivery, UiReceiver};
use aether_core::{Task, spawn, use_hook};
use std::{num::NonZeroUsize, task::Poll};

/// Attach one canonical receiver to the current component's lifetime.
///
/// Call unconditionally during rendering, following Aether's hook rules. The factory
/// and callback are captured on the first render only; use reactive handles inside
/// `apply` when it needs current application state. The factory runs on the UI thread
/// and must return the receiver for the actual required QualiaDB runtime. It does
/// not create a database, policy authority, or script executor.
///
/// The task is owned by this component. Unmounting the component, dropping its
/// VirtualDom, or cancelling the returned task drops the receiver and invalidates
/// pending runtime publications. For navigation/reload, cancel this task and mount
/// a new keyed component with a fresh runtime/receiver; cancellation alone does not
/// restart a hook in the same component. Committed host effects are not rolled back.
///
/// At most `max_per_turn` ready deliveries are applied before yielding to Aether's
/// scheduler. `apply` runs only on the UI owner thread and must itself remain
/// bounded. The normal native VirtualDom/event-loop polling must remain active.
/// This is independent of Document's JavaScript eval and never evaluates script.
///
/// When this is the only delivery consumer, `ui_channel(capacity, || {})` is valid:
/// its async receiver registers Aether's task waker, so no custom window callback
/// is required. A native platform event loop is still needed to drive that task.
pub fn use_qualia_delivery(
    create_receiver: impl FnOnce() -> UiReceiver,
    max_per_turn: NonZeroUsize,
    mut apply: impl FnMut(UiDelivery) + 'static,
) -> Task {
    use_hook(move || {
        let mut receiver = create_receiver();
        spawn(async move {
            loop {
                for _ in 0..max_per_turn.get() {
                    let Some(delivery) = receiver.next().await else {
                        return;
                    };
                    apply(delivery);
                }
                // Yield even when a producer keeps the bounded queue continuously
                // occupied. Wake outside queue locks through the task's own waker.
                let mut yielded = false;
                std::future::poll_fn(move |cx| {
                    if yielded {
                        Poll::Ready(())
                    } else {
                        yielded = true;
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                })
                .await;
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HostError, UiSender, ui_channel};
    use aether_core::{Element, NoOpMutations, ScopeId, VNode, VirtualDom};
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    #[derive(Clone)]
    struct Props {
        receiver: Rc<RefCell<Option<UiReceiver>>>,
        created: Rc<Cell<usize>>,
        values: Rc<RefCell<Vec<f64>>>,
        task: Rc<Cell<Option<Task>>>,
        owner: std::thread::ThreadId,
    }
    fn app(props: Props) -> Element {
        let task = use_qualia_delivery(
            move || {
                props.created.set(props.created.get() + 1);
                props
                    .receiver
                    .borrow_mut()
                    .take()
                    .expect("factory called twice")
            },
            NonZeroUsize::new(1).unwrap(),
            move |delivery| {
                assert_eq!(std::thread::current().id(), props.owner);
                let vibe::Value::F64(value) = delivery.value else {
                    panic!("unexpected canonical value")
                };
                props.values.borrow_mut().push(value);
            },
        );
        props.task.set(Some(task));
        VNode::empty()
    }
    fn fixture() -> (UiSender, Props, VirtualDom) {
        let (sender, receiver) = ui_channel(4, || {}).unwrap();
        let props = Props {
            receiver: Rc::new(RefCell::new(Some(receiver))),
            created: Rc::new(Cell::new(0)),
            values: Rc::new(RefCell::new(Vec::new())),
            task: Rc::new(Cell::new(None)),
            owner: std::thread::current().id(),
        };
        let mut dom = VirtualDom::new_with_props(app, props.clone());
        dom.rebuild_in_place();
        (sender, props, dom)
    }
    fn delivery(value: f64) -> UiDelivery {
        UiDelivery {
            value: vibe::Value::F64(value),
            receipts: Vec::new(),
        }
    }

    #[test]
    fn actual_aether_task_obeys_batch_bound_and_owner_thread() {
        let (sender, props, dom) = fixture();
        let epoch = sender.begin().unwrap();
        sender.publish(epoch, delivery(1.0)).unwrap();
        sender.publish(epoch, delivery(2.0)).unwrap();
        let task = props.task.get().unwrap();
        assert!(dom.in_scope(ScopeId::APP, || task.poll_now()).is_pending());
        assert_eq!(*props.values.borrow(), [1.0]);
        assert!(dom.in_scope(ScopeId::APP, || task.poll_now()).is_pending());
        assert_eq!(*props.values.borrow(), [1.0, 2.0]);
        drop(sender);
        assert!(dom.in_scope(ScopeId::APP, || task.poll_now()).is_ready());
    }

    #[test]
    fn component_rerender_reuses_receiver_and_dom_drop_closes_publication() {
        let (sender, props, mut dom) = fixture();
        let epoch = sender.begin().unwrap();
        let initial = props.task.get().unwrap();
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate(&mut NoOpMutations);
        assert_eq!(props.created.get(), 1);
        assert_eq!(props.task.get(), Some(initial));
        assert!(sender.current(epoch));
        drop(dom);
        assert!(!sender.current(epoch));
        assert!(matches!(
            sender.publish(epoch, delivery(3.0)),
            Err(HostError::Cancelled)
        ));
    }
}
