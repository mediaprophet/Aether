use std::cell::Cell;
use std::rc::Rc;

use refract::{
    AssertionKind, BitemporalStamp, ChallengeState, ContextualAssertionView, ContributionStage,
    ContributionStageView, DimensionOutcome, FairnessReviewView, InlinePath, InstrumentReadiness,
    QuinFieldLens, QuinSlot, QuinWindow, QuinWindowLens, Ui, WellFairDimension,
    contextual_assertion_badge, contribution_lifecycle_bar, fairness_review_panel,
    instrument_readiness_card, lens, paths_overlap,
};

#[test]
fn inline_path_is_copy_stack_allocated_and_prefix_comparable() {
    let mut p1 = InlinePath::new();
    assert!(p1.is_empty());
    assert_eq!(p1.len(), 0);

    p1.push(0);
    p1.push(3);
    p1.push(1);

    // Copy semantics: no heap allocation.
    let p2 = p1;
    assert_eq!(p1, p2);
    assert_eq!(p1.as_slice(), &[0, 3, 1]);
    assert_eq!(&*p2, &[0, 3, 1]);

    let mut parent = InlinePath::new();
    parent.push(0);
    parent.push(3);

    let mut sibling = InlinePath::new();
    sibling.push(0);
    sibling.push(4);

    assert!(paths_overlap(&parent, &p1));
    assert!(paths_overlap(&p1, &parent));
    assert!(!paths_overlap(&sibling, &p1));
}

#[test]
fn quin_slot_layout_parity_and_window_granularity() {
    assert_eq!(std::mem::size_of::<QuinSlot>(), 48);

    let slot0 = QuinSlot::new(101, 202, 303, 404, 505);
    assert!(slot0.verify_parity());

    let mut corrupted = slot0;
    corrupted.object ^= 1;
    assert!(!corrupted.verify_parity());
    corrupted.recompute_parity();
    assert!(corrupted.verify_parity());

    struct GraphState {
        window: QuinWindow<8>,
    }

    let mut win = QuinWindow::<8>::with_bitemporal_cursor(1_000, 1_000);
    win.insert(slot0, BitemporalStamp::new(500, 500));
    win.insert(
        QuinSlot::new(111, 222, 333, 444, 555),
        BitemporalStamp::new(1_500, 500),
    );

    assert_eq!(win.active_len(), 2);
    assert!(win.verify_all_parity());
    assert!(win.is_slot_visible(0));
    assert!(!win.is_slot_visible(1));
    assert_eq!(win.visible_count(), 1);

    let mut ui = Ui::new(GraphState { window: win });
    let window = lens!(GraphState => 0: window);
    let slot0_obj = window.slot(0).object();
    let slot1_obj = window.slot(1).object();
    let cursor_valid = window.cursor_valid_at();

    let runs0 = Rc::new(Cell::new(0));
    let runs1 = Rc::new(Cell::new(0));

    let r0 = runs0.clone();
    ui.effect(move |ctx| {
        let _ = *ctx.get(slot0_obj);
        r0.set(r0.get() + 1);
    });

    let r1 = runs1.clone();
    ui.effect(move |ctx| {
        let _ = *ctx.get(slot1_obj);
        r1.set(r1.get() + 1);
    });

    assert_eq!(runs0.get(), 1);
    assert_eq!(runs1.get(), 1);

    // Mutating slot 0 object only wakes slot 0 observer, not slot 1.
    ui.with(|ctx| {
        *ctx.write(slot0_obj) = 999;
    });
    assert_eq!(runs0.get(), 2);
    assert_eq!(runs1.get(), 1);

    // Advancing the bitemporal valid cursor does not spuriously wake slot object observers.
    ui.with(|ctx| {
        *ctx.write(cursor_valid) = 2_000;
    });
    assert_eq!(runs0.get(), 2);
    assert_eq!(runs1.get(), 1);
    assert_eq!(ui.get(window).visible_count(), 2);
}

#[test]
fn semantic_instrument_card_enforces_collect_not_equal_activate_and_soft_hold() {
    struct BenchState {
        readiness: InstrumentReadiness,
    }

    let mut ui = Ui::new(BenchState {
        readiness: InstrumentReadiness::CollectedInactive,
    });
    let readiness = lens!(BenchState => 0: readiness);

    let root = ui.mount(instrument_readiness_card(
        "wellfair-fairness-rights-review",
        readiness,
    ));
    let html = ui.render_to_string(root);
    assert!(html.contains("data-inspect-before-fetch=\"true\""));
    assert!(html.contains("data-collected=\"true\""));
    assert!(html.contains("data-active=\"false\""));
    assert!(html.contains("Collected (Inactive"));

    // Transition to Held with harsh "broken" wording: must be sanitized and never render "broken".
    ui.with(|ctx| {
        *ctx.write(readiness) = InstrumentReadiness::Held {
            why: "upstream schema is broken awaiting steward attestation",
        };
    });
    let held_html = ui.render_to_string(root);
    assert!(held_html.contains("data-state=\"held\""));
    assert!(!held_html.to_lowercase().contains("broken"));
    assert!(held_html.contains("Held (not yet ready):"));
    assert!(held_html.contains("held-pending-prerequisite"));

    // Activate explicitly.
    ui.with(|ctx| {
        *ctx.write(readiness) = InstrumentReadiness::ActiveReady;
    });
    let active_html = ui.render_to_string(root);
    assert!(active_html.contains("data-collected=\"true\""));
    assert!(active_html.contains("data-active=\"true\""));
}

#[test]
fn fairness_review_panel_renders_six_dimensions_without_single_score() {
    struct ReviewState {
        review: FairnessReviewView,
    }

    let initial = FairnessReviewView::new([
        DimensionOutcome::Satisfied,
        DimensionOutcome::Satisfied,
        DimensionOutcome::Held,
        DimensionOutcome::Satisfied,
        DimensionOutcome::Failed,
        DimensionOutcome::NotApplicable,
    ])
    .with_remedy(
        WellFairDimension::ResolutionRemedy,
        "Appoint independent dispute panel",
    );

    let mut ui = Ui::new(ReviewState { review: initial });
    let review = lens!(ReviewState => 0: review);
    let root = ui.mount(fairness_review_panel(review));

    let html = ui.render_to_string(root);
    assert!(html.contains("data-collapsed-score=\"forbidden\""));
    assert!(html.contains("data-all-satisfied=\"false\""));
    assert!(html.contains("data-dimension=\"legal-authority\""));
    assert!(html.contains("data-dimension=\"temporal-lifecycle\""));
    assert!(html.contains("Appoint independent dispute panel"));

    // Satisfy remaining dimensions and verify reactive update.
    ui.with(|ctx| {
        let mut r = *ctx.get(review);
        r.outcomes[WellFairDimension::EconomicMaterial as usize] = DimensionOutcome::Satisfied;
        r.outcomes[WellFairDimension::ResolutionRemedy as usize] = DimensionOutcome::Satisfied;
        *ctx.write(review) = r;
    });

    let updated = ui.render_to_string(root);
    assert!(updated.contains("data-all-satisfied=\"true\""));
}

#[test]
fn contextual_assertion_badge_keeps_epistemic_kinds_distinct() {
    struct EpistemicState {
        assertion: ContextualAssertionView,
    }

    let mut ui = Ui::new(EpistemicState {
        assertion: ContextualAssertionView {
            kind: AssertionKind::Allegation,
            challenge: ChallengeState::Restricted,
            independent_source_count: 1,
            summary: "Unverified delivery dispute claim",
        },
    });
    let assertion = lens!(EpistemicState => 0: assertion);
    let root = ui.mount(contextual_assertion_badge(assertion));

    let html = ui.render_to_string(root);
    assert!(html.contains("data-assertion-kind=\"allegation\""));
    assert!(html.contains("data-challenge-state=\"restricted\""));
    assert!(html.contains("Allegation (Unverified / Restricted Claim)"));
    assert!(html.contains("NOTICE: Restricted"));

    // Advance after due process to AdjudicatedFinding.
    ui.with(|ctx| {
        let mut a = *ctx.get(assertion);
        a.kind = AssertionKind::AdjudicatedFinding;
        a.challenge = ChallengeState::Unchallenged;
        a.independent_source_count = 3;
        *ctx.write(assertion) = a;
    });

    let updated = ui.render_to_string(root);
    assert!(updated.contains("data-assertion-kind=\"adjudicated-finding\""));
    assert!(updated.contains("Adjudicated Finding (Due-Process Determination)"));
    assert!(updated.contains("data-independent-sources=\"3\""));
}

#[test]
fn contribution_lifecycle_bar_enforces_pool_funded_not_equal_contributor_paid() {
    struct CoopState {
        lifecycle: ContributionStageView,
    }

    // Pool is 100% funded (50_000 cents), but contributor settlement is only 20_000 cents.
    let mut ui = Ui::new(CoopState {
        lifecycle: ContributionStageView {
            current_stage: ContributionStage::SettlementAllocation,
            target_threshold_cents: 50_000,
            pool_funded_cents: 50_000,
            contributor_settled_cents: 20_000,
            waived_obligation_cents: 0,
            unresolved_challenges: 0,
            human_transition_approved: true,
        },
    });
    let lifecycle = lens!(CoopState => 0: lifecycle);
    let root = ui.mount(contribution_lifecycle_bar(lifecycle));

    let html = ui.render_to_string(root);
    assert!(html.contains("data-pool-equals-paid=\"false\""));
    assert!(html.contains("data-transition-ready=\"false\""));
    assert!(html.contains("Pool Funded: 50000 / 50000 cents"));
    assert!(html.contains("Contributor Settled: 20000 / 50000 cents (Unpaid Gap: 30000 cents)"));

    // Settle the remaining 30_000 cents to the contributor with verified receipt.
    ui.with(|ctx| {
        let mut v = *ctx.get(lifecycle);
        v.contributor_settled_cents = 50_000;
        v.current_stage = ContributionStage::TransitionDecision;
        *ctx.write(lifecycle) = v;
    });

    let settled_html = ui.render_to_string(root);
    assert!(settled_html.contains("data-transition-ready=\"true\""));
    assert!(settled_html.contains("Contributor Settled: 50000 / 50000 cents (Unpaid Gap: 0 cents)"));
}
