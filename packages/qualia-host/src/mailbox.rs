// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
use crate::HostError;
use std::{
    collections::VecDeque,
    marker::PhantomData,
    rc::Rc,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};
use vibescript_abi::ExecutionReceiptHeader;

/// Canonical values and authoritative receipt headers; not a new UI wire schema.
#[derive(Clone, Debug)]
pub struct UiDelivery {
    pub value: vibe::Value,
    pub receipts: Vec<ExecutionReceiptHeader>,
}

struct State {
    epoch: u64,
    alive: bool,
    producer_alive: bool,
    capacity: usize,
    pending: VecDeque<UiDelivery>,
    waiter: Option<Waker>,
}

/// Single producer owned by the script runtime. Wake must only schedule native work.
pub struct UiSender {
    state: Arc<Mutex<State>>,
    wake: Arc<dyn Fn() + Send + Sync>,
}

/// Native UI owner. This handle deliberately cannot move between threads.
pub struct UiReceiver {
    state: Arc<Mutex<State>>,
    _native_thread: PhantomData<Rc<()>>,
}

/// Create on the native UI thread. Move the sender to the script owner if desired.
/// The wake callback should enqueue a native event, never mutate the DOM directly.
pub fn ui_channel(
    capacity: usize,
    wake: impl Fn() + Send + Sync + 'static,
) -> Result<(UiSender, UiReceiver), HostError> {
    if capacity == 0 {
        return Err(HostError::InvalidLimits);
    }
    let state = Arc::new(Mutex::new(State {
        epoch: 0,
        alive: true,
        producer_alive: true,
        capacity,
        pending: VecDeque::new(),
        waiter: None,
    }));
    Ok((
        UiSender {
            state: state.clone(),
            wake: Arc::new(wake),
        },
        UiReceiver {
            state,
            _native_thread: PhantomData,
        },
    ))
}

impl UiSender {
    pub(crate) fn begin(&self) -> Result<u64, HostError> {
        let state = self.state.lock().map_err(|_| HostError::Closed)?;
        if !state.alive {
            return Err(HostError::Closed);
        }
        if state.pending.len() == state.capacity {
            return Err(HostError::QueueFull);
        }
        Ok(state.epoch)
    }
    pub(crate) fn current(&self, epoch: u64) -> bool {
        self.state.lock().is_ok_and(|s| s.alive && s.epoch == epoch)
    }
    pub(crate) fn publish(&self, epoch: u64, delivery: UiDelivery) -> Result<(), HostError> {
        let waiter = {
            let mut state = self.state.lock().map_err(|_| HostError::Closed)?;
            if !state.alive || state.epoch != epoch {
                return Err(HostError::Cancelled);
            }
            if state.pending.len() == state.capacity {
                return Err(HostError::QueueFull);
            }
            state.pending.push_back(delivery);
            state.waiter.take()
        };
        // Waking can reenter the scheduler: never hold the mailbox lock here.
        if let Some(waiter) = waiter {
            waiter.wake();
        }
        (self.wake)();
        Ok(())
    }
}

impl UiReceiver {
    /// Poll from an owner-thread task. There is exactly one retained waiter.
    /// Producer shutdown drains existing results before returning `None`.
    pub fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<UiDelivery>> {
        // RawWaker clone/drop may execute scheduler code that reenters this mailbox.
        // Declare this before the guard so every early return unlocks before drop.
        let next_waiter = cx.waker().clone();
        let Ok(mut state) = self.state.lock() else {
            return Poll::Ready(None);
        };
        if let Some(delivery) = state.pending.pop_front() {
            return Poll::Ready(Some(delivery));
        }
        if !state.alive || !state.producer_alive {
            return Poll::Ready(None);
        }
        let previous_waiter = if state
            .waiter
            .as_ref()
            .is_none_or(|waiter| !waiter.will_wake(cx.waker()))
        {
            state.waiter.replace(next_waiter)
        } else {
            Some(next_waiter)
        };
        drop(state);
        drop(previous_waiter);
        Poll::Pending
    }

    /// Await from an Aether owner task; this future stays !Send with its receiver.
    /// Dropping the owner task drops the receiver and cancels subsequent publication.
    pub async fn next(&mut self) -> Option<UiDelivery> {
        std::future::poll_fn(|cx| self.poll_next(cx)).await
    }

    /// Drain at most `limit` entries on the native UI owner thread.
    pub fn drain(&mut self, limit: usize, mut apply: impl FnMut(UiDelivery)) {
        for _ in 0..limit {
            let next = self
                .state
                .lock()
                .ok()
                .and_then(|mut s| s.pending.pop_front());
            let Some(next) = next else { break };
            // Never invoke application code while holding the queue mutex.
            apply(next);
        }
    }
    /// Call on navigation, reload, or component replacement. Discard stale replies.
    pub fn invalidate(&mut self) {
        drop(invalidate_epoch(&self.state));
    }

    pub(crate) fn epoch_invalidator(&self) -> impl Fn() -> Vec<ExecutionReceiptHeader> + use<> {
        let state = Arc::downgrade(&self.state);
        move || {
            state
                .upgrade()
                .into_iter()
                .flat_map(|state| invalidate_epoch(&state))
                .flat_map(|delivery| delivery.receipts)
                .collect()
        }
    }
}

fn invalidate_epoch(state: &Mutex<State>) -> VecDeque<UiDelivery> {
    state
        .lock()
        .ok()
        .map(|mut state| {
            match state.epoch.checked_add(1) {
                Some(epoch) => state.epoch = epoch,
                None => state.alive = false,
            }
            std::mem::take(&mut state.pending)
        })
        .unwrap_or_default()
}

impl Drop for UiReceiver {
    fn drop(&mut self) {
        let retired = self.state.lock().ok().map(|mut state| {
            state.alive = false;
            (std::mem::take(&mut state.pending), state.waiter.take())
        });
        drop(retired);
    }
}

impl Drop for UiSender {
    fn drop(&mut self) {
        let waiter = self.state.lock().ok().and_then(|mut state| {
            state.producer_alive = false;
            state.waiter.take()
        });
        if let Some(waiter) = waiter {
            waiter.wake();
        }
    }
}
