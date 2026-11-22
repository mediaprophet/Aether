// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Component-owned polling of canonical authority graphs.
use crate::{GraphSnapshot, NativeGraphView};
use aether_core::{Task, spawn, use_hook};
use qualia_core_db::vibe_host::native_authority::graph::GraphError;
use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::Poll,
};

struct Owner {
    view: NativeGraphView,
    cancelled: Arc<AtomicBool>,
}
impl Owner {
    fn poll(&mut self) -> Result<Option<GraphSnapshot<'_>>, GraphError> {
        self.view.poll()
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        self.view.close();
    }
}

/// Poll one bounded graph snapshot for each application-owned tick.
///
/// Call unconditionally as an Aether hook. All inputs are captured only on first
/// render. `next_tick` returns false when its source closes; it must wait for real
/// native events/timers rather than manufacture continuous ready ticks. The hook
/// yields after each poll even if a tick is immediately ready. `apply` executes on
/// the component's native thread outside authority locks, and must stay bounded.
///
/// Supply the SAME cancellation token passed to native_graph_runtime. Cancelling
/// the task, unmounting the component or dropping the VirtualDom cancels subsequent
/// graph host calls and clears the view, even if this task was never polled. A
/// journal append already underway can still finish; reconcile its native receipt.
/// External cancellation is checked before reading and applying on each tick;
/// it closes the view and emits one Cancelled result. The token does not wake a
/// pending application tick future on its own.
/// Navigation/reload needs a fresh token and keyed component. This hook creates no
/// timer, daemon, script thread, authority or platform window.
///
/// Busy is retryable and preserves the prior view. An authentication/stale/closed
/// error requires clearing displayed private content; no earlier legitimate copy
/// can be retroactively erased by this adapter.
pub fn use_qualia_graph<F: Future<Output = bool> + 'static>(
    create_view: impl FnOnce() -> NativeGraphView,
    cancelled: Arc<AtomicBool>,
    mut next_tick: impl FnMut() -> F + 'static,
    mut apply: impl for<'a> FnMut(Result<Option<GraphSnapshot<'a>>, GraphError>) + 'static,
) -> Task {
    use_hook(move || {
        let mut owner = Owner {
            view: create_view(),
            cancelled,
        };
        spawn(async move {
            let cancellation = owner.cancelled.clone();
            while next_tick().await {
                if cancellation.load(Ordering::Acquire) {
                    break;
                }
                let result = owner.poll();
                if cancellation.load(Ordering::Acquire) {
                    drop(result);
                    break;
                }
                apply(result);
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
            owner.cancelled.store(true, Ordering::Release);
            owner.view.close();
            apply(Err(GraphError::Cancelled));
        })
    })
}
