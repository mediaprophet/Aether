// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Aether component subscriptions to Refract state projections.

use aether_core::{ReactiveContext, Subscribers, provide_context, use_hook};
use refract::{
    Field, InlinePath, Lens, QuinFieldLens, QuinSlot, QuinSlotLens, QuinWindow, QuinWindowLens,
};
use std::{cell::RefCell, rc::Rc};

struct RefractStoreInner<S: 'static> {
    ui: RefCell<refract::Ui<S>>,
    subscribers: RefCell<Vec<(InlinePath, Subscribers)>>,
}

/// Component-owned Refract state shared by reference with descendant scopes.
pub struct RefractStore<S: 'static>(Rc<RefractStoreInner<S>>);

impl<S> Clone for RefractStore<S> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

/// Create a Refract store once and provide it through the current Aether scope.
pub fn use_refract_store<S: 'static>(init: impl FnOnce() -> S) -> RefractStore<S> {
    use_hook(|| provide_context(RefractStore::new(init())))
}

impl<S: 'static> RefractStore<S> {
    /// Construct a store that can be passed through component props or context.
    pub fn new(state: S) -> Self {
        Self(Rc::new(RefractStoreInner {
            ui: RefCell::new(refract::Ui::new(state)),
            subscribers: RefCell::new(Vec::new()),
        }))
    }

    /// Read a projection and register it with the current reactive context.
    pub fn read<L: Lens<S>, R>(&self, lens: L, f: impl FnOnce(&L::Target) -> R) -> R {
        if let Some(context) = ReactiveContext::current() {
            let path = lens.path();
            let subscriptions = {
                let mut subscribers = self.0.subscribers.borrow_mut();
                if let Some((_, subscriptions)) = subscribers.iter().find(|(key, _)| *key == path) {
                    subscriptions.clone()
                } else {
                    let subscriptions = Subscribers::new();
                    subscribers.push((path, subscriptions.clone()));
                    subscriptions
                }
            };
            context.subscribe(subscriptions);
        }
        f(self.0.ui.borrow().get(lens))
    }

    /// Copy the current projection through its Clone implementation.
    pub fn get_cloned<L: Lens<S>>(&self, lens: L) -> L::Target
    where
        L::Target: Clone,
    {
        self.read(lens, Clone::clone)
    }

    /// Flush a write and notify scopes observing overlapping projections.
    pub fn write<L: Lens<S>>(&self, lens: L, f: impl FnOnce(&mut L::Target)) {
        let dirty_paths = {
            let mut ui = self.0.ui.borrow_mut();
            ui.with(|ctx| f(&mut *ctx.write(lens)));
            ui.take_dirty_paths()
        };
        let mut dirty_contexts = Vec::new();
        self.0
            .subscribers
            .borrow_mut()
            .retain(|(path, subscriptions)| {
                let dirty = dirty_paths
                    .iter()
                    .any(|written| refract::paths_overlap(path, written));
                let mut occupied = false;
                subscriptions.visit(|context| {
                    occupied = true;
                    if dirty && !dirty_contexts.contains(context) {
                        dirty_contexts.push(*context);
                    }
                });
                occupied
            });
        for context in dirty_contexts {
            context.mark_dirty();
        }
    }

    /// Read one slot using a stack-allocated lens.
    pub fn read_quin_slot<L, const N: usize>(&self, window: L, slot: usize) -> QuinSlot
    where
        L: Lens<S, Target = QuinWindow<N>>,
    {
        self.get_cloned(window.slot(slot))
    }

    /// Read a single word of one slot using a stack-allocated lens.
    /// The field index must be in 0..6.
    pub fn read_quin_field<L, const N: usize>(&self, window: L, slot: usize, field: u8) -> u64
    where
        L: Lens<S, Target = QuinWindow<N>>,
    {
        self.get_cloned(quin_field::<S, L, N>(window, slot, field))
    }

    /// Write one word of one slot. The caller owns parity maintenance.
    /// The field index must be in 0..6.
    pub fn write_quin_field<L, const N: usize>(&self, window: L, slot: usize, field: u8, value: u64)
    where
        L: Lens<S, Target = QuinWindow<N>>,
    {
        self.write(quin_field::<S, L, N>(window, slot, field), |word| {
            *word = value
        });
    }
}

fn quin_field<S: 'static, L, const N: usize>(
    window: L,
    slot: usize,
    field: u8,
) -> Field<QuinSlotLens<L, N>, QuinSlot, u64>
where
    L: Lens<S, Target = QuinWindow<N>>,
{
    let slot = window.slot(slot);
    match field {
        0 => slot.subject(),
        1 => slot.predicate(),
        2 => slot.object(),
        3 => slot.context(),
        4 => slot.metadata(),
        5 => slot.parity(),
        _ => panic!("Quin field index {field} is outside 0..6"),
    }
}

#[cfg(all(feature = "html", feature = "macro"))]
mod semantic {
    use crate::prelude::*;
    use refract::{
        AssertionEvidenceLevel, ContextualAssertionView, ContributionStageView, FairnessReviewView,
        InstrumentReadiness, WellFairDimension,
    };

    /// Render readiness, keeping discovery, collection and activation distinct.
    pub fn render_instrument_readiness_card(
        instrument_id: &str,
        readiness: InstrumentReadiness,
    ) -> Element {
        let status = readiness.status_code();
        let collected = readiness.is_collected();
        let active = readiness.is_active();
        let label = readiness.human_label();
        rsx! {
            section {
                class: "si-readiness-card",
                "data-instrument-id": "{instrument_id}",
                "data-inspect-before-fetch": "true",
                "data-state": "{status}",
                "data-collected": "{collected}",
                "data-active": "{active}",
                header { "{instrument_id}" }
                span { "{label}" }
            }
        }
    }

    /// Render every WellFair dimension and its own remedy.
    pub fn render_fairness_review_panel(review: &FairnessReviewView) -> Element {
        let all_satisfied = review.all_satisfied_or_na();
        rsx! {
            section {
                class: "wellfair-review-panel",
                "data-collapsed-score": "forbidden",
                "data-all-satisfied": "{all_satisfied}",
                for dimension in WellFairDimension::ALL {
                    div {
                        key: "{dimension.code()}",
                        class: "wellfair-dimension-row",
                        "data-dimension": "{dimension.code()}",
                        "data-outcome": "{review.outcome(dimension).code()}",
                        span { "{dimension.title()}" }
                        span { "{review.outcome(dimension).label()}" }
                        if !review.remedy(dimension).is_empty() {
                            span { "{review.remedy(dimension)}" }
                        }
                    }
                }
            }
        }
    }

    /// Render the statement kind and challenge notice.
    pub fn render_contextual_assertion_badge(assertion: &ContextualAssertionView) -> Element {
        render_contextual_assertion_badge_with_evidence(assertion, None)
    }

    /// Include the stated basis of the assertion when the host provides it.
    pub fn render_contextual_assertion_badge_with_evidence(
        assertion: &ContextualAssertionView,
        evidence: Option<AssertionEvidenceLevel>,
    ) -> Element {
        let kind = assertion.kind;
        let challenge = assertion.challenge;
        let independent_sources = assertion.independent_source_count;
        let summary = assertion.summary;
        rsx! {
            article {
                class: "contextual-assertion-badge",
                "data-assertion-kind": "{kind.code()}",
                "data-challenge-state": "{challenge.code()}",
                "data-independent-sources": "{independent_sources}",
                "data-evidence-level": evidence.map(AssertionEvidenceLevel::code),
                span { "{kind.badge_label()}" }
                span { "{challenge.notice()}" }
                if let Some(level) = evidence {
                    span { "{level.label()}" }
                }
                p { "{summary}" }
            }
        }
    }

    /// Render project-pool funding and contributor settlements independently.
    pub fn render_contribution_lifecycle_bar(view: &ContributionStageView) -> Element {
        let stage = view.current_stage.code();
        let transition = view.can_transition_to_commons();
        let pool = view.pool_funded_cents;
        let paid = view.contributor_settled_cents;
        let target = view.target_threshold_cents;
        let gap = view.unpaid_settled_gap_cents();
        rsx! {
            section {
                class: "contribution-lifecycle-bar",
                "data-pool-equals-paid": "false",
                "data-stage": "{stage}",
                "data-transition-ready": "{transition}",
                "data-pool-funded-minor": "{pool}",
                "data-contributor-paid-minor": "{paid}",
                div { "data-role": "pool-funded", "Pool Funded {pool} / {target} cents" }
                div { "data-role": "contributor-settled", "Contributor Settled {paid} / {target} cents ({gap} cents outstanding)" }
            }
        }
    }
}

#[cfg(all(feature = "html", feature = "macro"))]
pub use semantic::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::{ScopeId, VNode, VirtualDom};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn a_quin_field_write_invalidates_only_overlapping_readers() {
        let mut dom = VirtualDom::new(VNode::empty);
        dom.rebuild_in_place();
        dom.in_scope(ScopeId::APP, || {
            let store = RefractStore::new(QuinWindow::<8>::default());
            let lens = refract::Root::new();
            let notifications = [Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0))];
            let contexts = notifications.clone().map(|count| {
                ReactiveContext::new_with_callback(
                    move || {
                        count.fetch_add(1, Ordering::Relaxed);
                    },
                    ScopeId::APP,
                    std::panic::Location::caller(),
                )
            });
            contexts[0].run_in(|| store.read_quin_slot(lens, 0));
            contexts[1].run_in(|| store.read_quin_slot(lens, 1));
            store.write_quin_field(lens, 0, 2, 42);
            assert_eq!(notifications[0].load(Ordering::Relaxed), 1);
            assert_eq!(notifications[1].load(Ordering::Relaxed), 0);
            assert_eq!(store.read_quin_field(lens, 0, 2), 42);

            contexts[0].reset_and_run_in(|| store.read_quin_slot(lens, 1));
            store.write_quin_field(lens, 0, 2, 43);
            assert_eq!(notifications[0].load(Ordering::Relaxed), 1);
            assert_eq!(notifications[1].load(Ordering::Relaxed), 0);
        });
    }

    #[cfg(all(feature = "html", feature = "macro"))]
    #[test]
    fn semantic_rsx_preserves_readiness_evidence_and_settlement() {
        use aether_core::{AttributeValue, DynamicNode, VNodeChild};
        use refract::{
            AssertionEvidenceLevel, AssertionKind, ChallengeState, ContextualAssertionView,
            ContributionStage, ContributionStageView, DimensionOutcome, FairnessReviewView,
            InstrumentReadiness, WellFairDimension,
        };
        #[derive(Default)]
        struct Output {
            attributes: Vec<(String, String)>,
            text: String,
        }
        impl Output {
            fn has(&self, name: &str, value: &str) -> bool {
                self.attributes.iter().any(|(n, v)| n == name && v == value)
            }

            fn visit(&mut self, child: VNodeChild<'_>) {
                match child {
                    VNodeChild::Element(element) => {
                        for (name, value, _) in element.static_attributes() {
                            self.attributes.push((name.into(), value.into()));
                        }
                        for slot in element.dynamic_anchors().flat_map(|anchor| anchor.attrs()) {
                            for attr in slot.attrs() {
                                if let AttributeValue::Text(value) = &attr.value {
                                    self.attributes.push((attr.name.into(), value.clone()));
                                }
                            }
                        }
                        for child in element.children() {
                            self.visit(child);
                        }
                    }
                    VNodeChild::Text(text) => self.text.push_str(text.text()),
                    VNodeChild::Dynamic(anchor) => {
                        for node in anchor.nodes() {
                            match &*node {
                                DynamicNode::Text(text) => self.text.push_str(&text.value),
                                DynamicNode::Fragment(nodes) => {
                                    for node in nodes {
                                        for child in node.children() {
                                            self.visit(child);
                                        }
                                    }
                                }
                                DynamicNode::Component(_) => {}
                            }
                        }
                    }
                }
            }
        }
        let inspect = |element: aether_core::Element| {
            let mut output = Output::default();
            for child in element.unwrap().children() {
                output.visit(child);
            }
            output
        };

        let mut dom = VirtualDom::new(VNode::empty);
        dom.rebuild_in_place();
        dom.in_scope(ScopeId::APP, || {
            let card = inspect(render_instrument_readiness_card(
                "SI-01",
                InstrumentReadiness::Held {
                    why: "BROKEN input",
                },
            ));
            assert!(card.has("data-inspect-before-fetch", "true"));
            assert!(card.has("data-collected", "true"));
            assert!(card.has("data-active", "false"));
            assert!(!card.text.to_ascii_lowercase().contains("broken"));

            let review = FairnessReviewView::new([DimensionOutcome::Failed; 6])
                .with_remedy(WellFairDimension::ResolutionRemedy, "Review again");
            let panel = inspect(render_fairness_review_panel(&review));
            assert!(panel.has("data-collapsed-score", "forbidden"));
            assert!(
                WellFairDimension::ALL
                    .iter()
                    .all(|dim| panel.has("data-dimension", dim.code()))
            );
            assert!(panel.text.contains("Review again"));

            let assertion = ContextualAssertionView {
                kind: AssertionKind::Allegation,
                challenge: ChallengeState::Restricted,
                independent_source_count: 0,
                summary: "Contested assertion",
            };
            let badge = inspect(render_contextual_assertion_badge(&assertion));
            assert!(badge.has("data-assertion-kind", "allegation"));
            assert!(badge.has("data-challenge-state", "restricted"));
            let simulated = inspect(render_contextual_assertion_badge_with_evidence(
                &assertion,
                Some(AssertionEvidenceLevel::Simulated),
            ));
            assert!(simulated.has("data-evidence-level", "simulated"));
            assert!(simulated.has("data-challenge-state", "restricted"));

            let contribution = ContributionStageView {
                current_stage: ContributionStage::TransitionDecision,
                target_threshold_cents: 100,
                pool_funded_cents: 100,
                contributor_settled_cents: 0,
                waived_obligation_cents: 0,
                unresolved_challenges: 0,
                human_transition_approved: true,
            };
            let lifecycle = inspect(render_contribution_lifecycle_bar(&contribution));
            assert!(lifecycle.has("data-pool-equals-paid", "false"));
            assert!(lifecycle.has("data-pool-funded-minor", "100"));
            assert!(lifecycle.has("data-contributor-paid-minor", "0"));
            assert!(lifecycle.has("data-transition-ready", "false"));
        });
    }
}
