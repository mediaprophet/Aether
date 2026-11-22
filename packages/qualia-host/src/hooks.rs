// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Declarative UI adapters over the adopted QualiaDB host contracts.

use crate::{
    AuthoringBridgeError, GraphSnapshot, HostError, NativeAuthoringBridge, NativeAuthoringView,
    NativeGraphView, UiDelivery, UiReceiver,
    native_ui::{use_qualia_delivery, use_qualia_graph},
};
use aether_core::{spawn, try_consume_context, use_drop, use_hook};
use aether_signals::{ReadableExt, Signal, WritableExt};
use qualia_core_db::vibe_host::native_authority::{
    NativeAuthority, SourceReviewRequest, WorkflowEvidence,
    graph::{GraphError, GraphRead},
};
use std::{
    cell::{Cell, RefCell},
    future::Future,
    num::NonZeroUsize,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::Poll,
};
use vibescript_abi::ExecutionReceiptHeader;

/// Host-adopted authority and optional native services for a component tree.
#[derive(Clone)]
pub struct QualiaHostContext {
    pub authority: Arc<NativeAuthority>,
    pub authoring: Option<NativeAuthoringBridge>,
    pub action_dispatcher: Option<Arc<dyn VibeActionDispatcher>>,
}

impl QualiaHostContext {
    pub fn new(authority: Arc<NativeAuthority>) -> Self {
        Self {
            authority,
            authoring: None,
            action_dispatcher: None,
        }
    }
}

pub fn use_qualia_context() -> QualiaHostContext {
    try_use_qualia_context().expect("QualiaHostContext must be provided by the host application")
}

pub fn try_use_qualia_context() -> Option<QualiaHostContext> {
    try_consume_context()
}

/// Enqueue work on a host-owned script runner and return its admitted epoch.
pub trait VibeActionDispatcher: Send + Sync + 'static {
    fn dispatch_action(
        &self,
        program_id: &str,
        entry_fn: &str,
        args: Vec<vibe::Value>,
        cancel_token: Arc<AtomicBool>,
    ) -> Result<u64, HostError>;
}

#[derive(Clone, Debug)]
pub enum VibeActionState {
    Idle,
    Running {
        epoch: u64,
    },
    Delivered(UiDelivery),
    Failed(String),
    NeedsReconcile {
        cause: String,
        committed_receipts: Vec<ExecutionReceiptHeader>,
    },
    Cancelled,
}

type InvalidateEpoch = Box<dyn Fn() -> Vec<ExecutionReceiptHeader>>;

#[derive(Default)]
struct ActionLifetime {
    token: RefCell<Option<Arc<AtomicBool>>>,
    invalidate: RefCell<Option<InvalidateEpoch>>,
    mounted: Cell<bool>,
}

impl ActionLifetime {
    fn cancel(&self) -> Vec<ExecutionReceiptHeader> {
        if let Some(token) = self.token.borrow_mut().take() {
            token.store(true, Ordering::Release);
        }
        self.invalidate
            .borrow()
            .as_ref()
            .map_or_else(Vec::new, |invalidate| invalidate())
    }
}

#[derive(Clone)]
pub struct VibeAction {
    pub state: Signal<VibeActionState>,
    dispatcher: Option<Arc<dyn VibeActionDispatcher>>,
    lifetime: Rc<ActionLifetime>,
}

impl VibeAction {
    pub fn dispatch(
        &mut self,
        program_id: &str,
        entry_fn: &str,
        args: Vec<vibe::Value>,
    ) -> Result<u64, HostError> {
        if !self.lifetime.mounted.get() {
            return Err(HostError::Closed);
        }
        if matches!(
            &*self.state.peek(),
            VibeActionState::Running { .. } | VibeActionState::NeedsReconcile { .. }
        ) {
            return Err(HostError::QueueFull);
        }
        let token = Arc::new(AtomicBool::new(false));
        let dispatcher = self
            .dispatcher
            .as_ref()
            .ok_or(HostError::MissingQualiaFoundation);
        match dispatcher
            .and_then(|d| d.dispatch_action(program_id, entry_fn, args, Arc::clone(&token)))
        {
            Ok(epoch) => {
                *self.lifetime.token.borrow_mut() = Some(token);
                self.state.set(VibeActionState::Running { epoch });
                Ok(epoch)
            }
            Err(error) => {
                self.apply_host_error(&error);
                Err(error)
            }
        }
    }

    pub fn cancel(&mut self) {
        if self.lifetime.mounted.get()
            && matches!(&*self.state.peek(), VibeActionState::Running { .. })
        {
            let committed_receipts = self.lifetime.cancel();
            if committed_receipts.is_empty() {
                self.state.set(VibeActionState::Cancelled);
            } else {
                self.state.set(VibeActionState::NeedsReconcile {
                    cause: "Action cancelled after a result was committed".into(),
                    committed_receipts,
                });
            }
        }
    }

    /// Release the reconciliation state once the host has retained its receipts.
    pub fn acknowledge_reconciliation(&mut self) {
        if self.lifetime.mounted.get()
            && matches!(&*self.state.peek(), VibeActionState::NeedsReconcile { .. })
        {
            self.state.set(VibeActionState::Idle);
        }
    }

    /// Apply errors reported by the host-owned execution path on the UI thread.
    pub fn record_host_error(&mut self, err: HostError) {
        if self.lifetime.mounted.get() {
            self.apply_host_error(&err);
        }
    }

    fn apply_host_error(&mut self, error: &HostError) {
        let mut receipts = self.lifetime.cancel();
        let previous = match &*self.state.peek() {
            VibeActionState::NeedsReconcile {
                cause,
                committed_receipts,
            } => Some((cause.clone(), committed_receipts.clone())),
            _ => None,
        };
        if let Some((_, previous_receipts)) = &previous {
            append_receipts(&mut receipts, previous_receipts);
        }
        let state = match error {
            HostError::Reconcile {
                cause,
                committed_receipts,
            } => {
                append_receipts(&mut receipts, committed_receipts);
                VibeActionState::NeedsReconcile {
                    cause: cause.to_string(),
                    committed_receipts: receipts,
                }
            }
            _ if previous.is_some() || !receipts.is_empty() => VibeActionState::NeedsReconcile {
                cause: previous.map_or_else(|| error.to_string(), |(cause, _)| cause),
                committed_receipts: receipts,
            },
            HostError::Cancelled => VibeActionState::Cancelled,
            _ => VibeActionState::Failed(error.to_string()),
        };
        self.state.set(state);
    }
}

fn append_receipts(output: &mut Vec<ExecutionReceiptHeader>, receipts: &[ExecutionReceiptHeader]) {
    for receipt in receipts {
        if !output.contains(receipt) {
            output.push(*receipt);
        }
    }
}

pub fn use_vibe_action(
    create_receiver: impl FnOnce() -> UiReceiver,
    max_per_turn: NonZeroUsize,
) -> VibeAction {
    let lifetime = use_hook(|| {
        Rc::new(ActionLifetime {
            mounted: Cell::new(true),
            ..ActionLifetime::default()
        })
    });
    let mut state = use_hook(|| Signal::new(VibeActionState::Idle));
    let owner = Rc::clone(&lifetime);
    use_drop(move || {
        owner.mounted.set(false);
        drop(owner.cancel());
    });
    let receiver_owner = Rc::clone(&lifetime);
    let apply_owner = Rc::clone(&lifetime);
    use_qualia_delivery(
        move || {
            let receiver = create_receiver();
            *receiver_owner.invalidate.borrow_mut() = Some(Box::new(receiver.epoch_invalidator()));
            receiver
        },
        max_per_turn,
        move |delivery| {
            if apply_owner.token.borrow_mut().take().is_some() {
                state.set(VibeActionState::Delivered(delivery));
            }
        },
    );
    VibeAction {
        state,
        dispatcher: try_use_qualia_context().and_then(|ctx| ctx.action_dispatcher),
        lifetime,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectedGraphSnapshot<T> {
    pub read: GraphRead,
    pub skipped_revisions: u64,
    pub data: T,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphSubscriptionStatus {
    Waiting,
    Current,
    BusyPreserved,
    Unavailable(GraphError),
}

pub struct QualiaSubscription<T: 'static> {
    pub snapshot: Signal<Option<ProjectedGraphSnapshot<T>>>,
    pub status: Signal<GraphSubscriptionStatus>,
}

impl<T: 'static> QualiaSubscription<T> {
    fn apply(&mut self, result: Result<Option<ProjectedGraphSnapshot<T>>, GraphError>) {
        match result {
            Ok(Some(snapshot)) => {
                self.snapshot.set(Some(snapshot));
                self.status.set(GraphSubscriptionStatus::Current);
            }
            Ok(None) => {
                if self.snapshot.peek().is_some() {
                    self.status.set(GraphSubscriptionStatus::Current);
                }
            }
            Err(GraphError::Busy) => self.status.set(GraphSubscriptionStatus::BusyPreserved),
            Err(error) => {
                self.snapshot.set(None);
                self.status.set(GraphSubscriptionStatus::Unavailable(error));
            }
        }
    }
}

pub fn use_qualia_subscription<T: 'static, F: Future<Output = bool> + 'static>(
    create_view: impl FnOnce() -> NativeGraphView,
    cancelled: Arc<AtomicBool>,
    next_tick: impl FnMut() -> F + 'static,
    mut project: impl for<'a> FnMut(&GraphSnapshot<'a>) -> T + 'static,
) -> QualiaSubscription<T> {
    let snapshot = use_hook(|| Signal::new(None));
    let status = use_hook(|| Signal::new(GraphSubscriptionStatus::Waiting));
    let mut apply = QualiaSubscription { snapshot, status };
    use_qualia_graph(create_view, cancelled, next_tick, move |result| {
        apply.apply(result.map(|snapshot| {
            snapshot.map(|snapshot| ProjectedGraphSnapshot {
                read: snapshot.read,
                skipped_revisions: snapshot.skipped_revisions,
                data: project(&snapshot),
            })
        }));
    });
    QualiaSubscription { snapshot, status }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthoringBridgeStatus {
    Waiting,
    Current,
    Unavailable(String),
    Closed,
}

#[derive(Clone)]
pub struct AuthoringBridge {
    owner: Rc<RefCell<Option<NativeAuthoringView>>>,
    pub evidence: Signal<Option<WorkflowEvidence>>,
    pub last_review: Signal<Option<SourceReviewRequest>>,
    pub status: Signal<AuthoringBridgeStatus>,
}

impl AuthoringBridge {
    fn clear(&self, status: AuthoringBridgeStatus) {
        let (mut evidence, mut last_review, mut signal) =
            (self.evidence, self.last_review, self.status);
        evidence.set(None);
        last_review.set(None);
        signal.set(status);
    }

    pub fn request_source_review(
        &self,
        id: &str,
        evidence_ref: &str,
    ) -> Result<SourceReviewRequest, AuthoringBridgeError> {
        let result = self
            .owner
            .borrow_mut()
            .as_mut()
            .ok_or(AuthoringBridgeError::Closed)
            .and_then(|view| view.request_source_review(id, evidence_ref));
        match &result {
            Ok(review) => {
                let mut signal = self.last_review;
                signal.set(Some(review.clone()));
            }
            Err(error) if self.owner.borrow().is_some() => {
                self.clear(AuthoringBridgeStatus::Unavailable(error.to_string()));
            }
            Err(_) => {}
        }
        result
    }
}

pub fn use_authoring_bridge<F: Future<Output = bool> + 'static>(
    create_view: impl FnOnce() -> NativeAuthoringView,
    mut next_tick: impl FnMut() -> F + 'static,
) -> AuthoringBridge {
    let bridge = use_hook(|| AuthoringBridge {
        owner: Rc::new(RefCell::new(Some(create_view()))),
        evidence: Signal::new(None),
        last_review: Signal::new(None),
        status: Signal::new(AuthoringBridgeStatus::Waiting),
    });
    let owner = Rc::clone(&bridge.owner);
    use_drop(move || {
        owner.borrow_mut().take();
    });
    let apply = bridge.clone();
    use_hook(move || {
        spawn(async move {
            loop {
                let result = {
                    let mut owner = apply.owner.borrow_mut();
                    let Some(view) = owner.as_mut() else {
                        return;
                    };
                    view.refresh().and_then(|()| view.take_current())
                };
                match result {
                    Ok(snapshot) => {
                        let mut evidence = apply.evidence;
                        let mut status = apply.status;
                        evidence.set(snapshot);
                        status.set(AuthoringBridgeStatus::Current);
                    }
                    Err(error) => {
                        apply.clear(AuthoringBridgeStatus::Unavailable(error.to_string()))
                    }
                }
                if !next_tick().await {
                    apply.owner.borrow_mut().take();
                    apply.clear(AuthoringBridgeStatus::Closed);
                    return;
                }
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
    });
    bridge
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::{ScopeId, VNode, VirtualDom};

    #[test]
    fn graph_busy_preserves_and_authentication_clears_the_projection() {
        let mut dom = VirtualDom::new(VNode::empty);
        dom.rebuild_in_place();
        dom.in_scope(ScopeId::APP, || {
            let mut subscription = QualiaSubscription {
                snapshot: Signal::new(None),
                status: Signal::new(GraphSubscriptionStatus::Waiting),
            };
            subscription.apply(Ok(Some(ProjectedGraphSnapshot {
                read: GraphRead {
                    context: 1,
                    revision: 2,
                    written: 1,
                },
                skipped_revisions: 0,
                data: "private",
            })));
            subscription.apply(Err(GraphError::Busy));
            assert_eq!(
                *subscription.status.peek(),
                GraphSubscriptionStatus::BusyPreserved
            );
            assert_eq!(
                subscription.snapshot.peek().as_ref().unwrap().data,
                "private"
            );
            subscription.apply(Err(GraphError::Authentication));
            assert!(subscription.snapshot.peek().is_none());
            assert_eq!(
                *subscription.status.peek(),
                GraphSubscriptionStatus::Unavailable(GraphError::Authentication)
            );
        });
    }

    fn committed_receipt() -> ExecutionReceiptHeader {
        ExecutionReceiptHeader {
            ticket_id: 1,
            actor_did_hash: 2,
            provider_iri_hash: 3,
            func_iri_hash: 4,
            stable_id: vibescript_abi::StableFuncId::new(1, 1),
            mode: vibescript_abi::ExecutionMode::GovernedApplication,
            status: vibescript_abi::CompletionStatus::Committed,
            simulated: false,
            reserved: [0; 5],
            policy_generation: 1,
            registry_generation: vibescript_abi::RegistryGeneration(1),
            ticks_consumed: 0,
            output_bytes_written: 0,
            trap_code: 0,
            failure_stage: 0,
            input_output_fold_hash: 0,
            lamport_timestamp: 0,
        }
    }

    #[test]
    fn cancel_retains_receipts_from_a_delivery_already_in_the_mailbox() {
        let mut dom = VirtualDom::new(VNode::empty);
        dom.rebuild_in_place();
        dom.in_scope(ScopeId::APP, || {
            let (sender, receiver) = crate::ui_channel(1, || {}).unwrap();
            let epoch = sender.begin().unwrap();
            let token = Arc::new(AtomicBool::new(false));
            let receipt = committed_receipt();
            sender.publish(epoch, UiDelivery {
                value: vibe::Value::F64(1.0),
                receipts: vec![receipt],
            }).unwrap();
            let mut action = VibeAction {
                state: Signal::new(VibeActionState::Running { epoch }),
                lifetime: Rc::new(ActionLifetime {
                    token: RefCell::new(Some(Arc::clone(&token))),
                    invalidate: RefCell::new(Some(Box::new(receiver.epoch_invalidator()))),
                    mounted: Cell::new(true),
                }),
                dispatcher: None,
            };
            action.cancel();
            assert!(token.load(Ordering::Acquire));
            assert!(!sender.current(epoch));
            assert!(matches!(&*action.state.peek(),
                VibeActionState::NeedsReconcile { committed_receipts, .. } if committed_receipts == &[receipt]));
        });
    }

    #[test]
    fn reconcile_retains_the_committed_receipts_even_after_cancellation() {
        let mut dom = VirtualDom::new(VNode::empty);
        dom.rebuild_in_place();
        dom.in_scope(ScopeId::APP, || {
            let receipt = committed_receipt();
            let mut action = VibeAction {
                state: Signal::new(VibeActionState::Running { epoch: 1 }),
                lifetime: Rc::new(ActionLifetime { mounted: Cell::new(true), ..Default::default() }),
                dispatcher: None,
            };
            action.record_host_error(HostError::Reconcile {
                cause: Box::new(HostError::Closed), committed_receipts: vec![receipt],
            });
            action.cancel();
            assert!(matches!(&*action.state.peek(), VibeActionState::NeedsReconcile { committed_receipts, .. }
                if committed_receipts == &[receipt]));
        });
    }
}
