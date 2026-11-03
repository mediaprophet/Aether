//! First-Class Semantic Instrument, Contextual Assertion, and Cooperative
//! Project Lifecycle UI Primitives for Refract and Aether.
//!
//! Enforces `collect != activate`, `inspect before fetch`, soft `Held { why }`
//! copy, 6-dimension WellFair reviews, epistemic separation (`Observation` !=
//! `Inference` != `Allegation` != `Attestation` != `AdjudicatedFinding`), and
//! `Pool Funded != Contributor Paid`.

use crate::dom::{El, dyn_text, el, text};
use crate::lens::Lens;

/// Lifecycle and readiness state of a Semantic Instrument (`SI-01`..`SI-06`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InstrumentReadiness {
    /// Manifest can be inspected, but the instrument bundle is not yet collected.
    Uncollected,
    /// Collected into local store, but not yet activated (`collect != activate`).
    CollectedInactive,
    /// Collected, verified, and actively ready to evaluate.
    ActiveReady,
    /// Temporarily paused by steward or user policy.
    Suspended,
    /// Explicitly revoked by governance or issuer receipt.
    Revoked,
    /// Held pending a prerequisite or input condition (never labelled "broken").
    Held {
        /// Human-readable reason explaining why execution is held.
        why: &'static str,
    },
}

impl InstrumentReadiness {
    /// Return `true` if the instrument package has been collected locally.
    #[inline]
    pub const fn is_collected(&self) -> bool {
        matches!(self, Self::CollectedInactive | Self::ActiveReady | Self::Suspended | Self::Held { .. })
    }

    /// Return `true` only when the instrument is actively enabled (`collect != activate`).
    #[inline]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::ActiveReady)
    }

    /// Manifest inspection is always permitted prior to collection or fetch.
    #[inline]
    pub const fn can_inspect_manifest(&self) -> bool {
        true
    }

    /// Canonical machine-readable status code.
    #[inline]
    pub const fn status_code(&self) -> &'static str {
        match self {
            Self::Uncollected => "uncollected",
            Self::CollectedInactive => "collected-inactive",
            Self::ActiveReady => "active-ready",
            Self::Suspended => "suspended",
            Self::Revoked => "revoked",
            Self::Held { .. } => "held",
        }
    }

    /// Sanitize any harsh "broken" wording into soft `held / not yet ready` phrasing.
    pub fn sanitize_hold_reason(raw: &str) -> String {
        let mut out = String::with_capacity(raw.len());
        for word in raw.split_whitespace() {
            if !out.is_empty() {
                out.push(' ');
            }
            let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric());
            if cleaned.eq_ignore_ascii_case("broken") {
                out.push_str("held-pending-prerequisite");
            } else {
                out.push_str(word);
            }
        }
        if out.is_empty() { "awaiting prerequisite input".to_string() } else { out }
    }

    /// Human-facing readiness label that never renders "broken".
    pub fn human_label(&self) -> String {
        match self {
            Self::Uncollected => "Uncollected (Inspect Manifest Before Fetch)".to_string(),
            Self::CollectedInactive => "Collected (Inactive — Activate Explicitly)".to_string(),
            Self::ActiveReady => "Active & Ready".to_string(),
            Self::Suspended => "Suspended".to_string(),
            Self::Revoked => "Revoked".to_string(),
            Self::Held { why } => format!("Held (not yet ready): {}", Self::sanitize_hold_reason(why)),
        }
    }
}

/// Build a reactive Semantic Instrument readiness card enforcing `collect != activate`.
pub fn instrument_readiness_card<S, L>(instrument_id: &'static str, readiness_lens: L) -> El<S>
where
    S: 'static,
    L: Lens<S, Target = InstrumentReadiness>,
{
    el("section")
        .attr("class", "si-readiness-card")
        .attr("data-instrument-id", instrument_id)
        .attr("data-inspect-before-fetch", "true")
        .dyn_attr("data-state", move |ctx| ctx.get(readiness_lens).status_code().to_string())
        .dyn_attr("data-collected", move |ctx| ctx.get(readiness_lens).is_collected().to_string())
        .dyn_attr("data-active", move |ctx| ctx.get(readiness_lens).is_active().to_string())
        .child(el("header").attr("class", "si-instrument-id").child(text(instrument_id)))
        .child(el("span").attr("class", "si-readiness-label").child(dyn_text(move |ctx| ctx.get(readiness_lens).human_label())))
}

/// The 6 canonical dimensions of the WellFair Fairness & Rights Review.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum WellFairDimension {
    /// Dimension 1: Legal & Normative Authority (`LegalAuthority`).
    LegalAuthority = 0,
    /// Dimension 2: Ethical & Human Agency (`EthicalHumanAgency`).
    EthicalHumanAgency = 1,
    /// Dimension 3: Economic & Material Fairness (`EconomicMaterial`).
    EconomicMaterial = 2,
    /// Dimension 4: Social & Cooperative Participation (`SocialParticipation`).
    SocialParticipation = 3,
    /// Dimension 5: Contestability, Resolution & Remedy (`ResolutionRemedy`).
    ResolutionRemedy = 4,
    /// Dimension 6: Temporal & Lifecycle Stewardship (`TemporalLifecycle`).
    TemporalLifecycle = 5,
}

impl WellFairDimension {
    /// All 6 required WellFair dimensions in canonical order.
    pub const ALL: [Self; 6] = [
        Self::LegalAuthority,
        Self::EthicalHumanAgency,
        Self::EconomicMaterial,
        Self::SocialParticipation,
        Self::ResolutionRemedy,
        Self::TemporalLifecycle,
    ];

    /// Machine-readable dimension code.
    #[inline]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::LegalAuthority => "legal-authority",
            Self::EthicalHumanAgency => "ethical-human-agency",
            Self::EconomicMaterial => "economic-material",
            Self::SocialParticipation => "social-participation",
            Self::ResolutionRemedy => "resolution-remedy",
            Self::TemporalLifecycle => "temporal-lifecycle",
        }
    }

    /// Human-readable dimension title.
    #[inline]
    pub const fn title(&self) -> &'static str {
        match self {
            Self::LegalAuthority => "Legal & Normative Authority",
            Self::EthicalHumanAgency => "Ethical & Human Agency",
            Self::EconomicMaterial => "Economic & Material Fairness",
            Self::SocialParticipation => "Social & Cooperative Participation",
            Self::ResolutionRemedy => "Resolution & Remedy",
            Self::TemporalLifecycle => "Temporal & Lifecycle Stewardship",
        }
    }
}

/// Per-dimension outcome in a WellFair Fairness & Rights Review.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DimensionOutcome {
    /// Dimension requirements are satisfied with verified evidence.
    Satisfied,
    /// Dimension requirements failed validation; remedy required.
    Failed,
    /// Dimension evaluation is held awaiting prerequisite attestations.
    Held,
    /// Dimension is not applicable to the bounded scope.
    NotApplicable,
}

impl DimensionOutcome {
    /// Machine-readable outcome code.
    #[inline]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Satisfied => "satisfied",
            Self::Failed => "failed",
            Self::Held => "held",
            Self::NotApplicable => "not-applicable",
        }
    }

    /// Human-readable outcome badge label.
    #[inline]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Satisfied => "Satisfied",
            Self::Failed => "Failed (Remedy Required)",
            Self::Held => "Held (Pending Prerequisite)",
            Self::NotApplicable => "Not Applicable",
        }
    }
}

/// Multi-dimensional WellFair Fairness & Rights Review state (forbids single-number scores).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FairnessReviewView {
    /// Per-dimension outcomes indexed by [`WellFairDimension`].
    pub outcomes: [DimensionOutcome; 6],
    /// Per-dimension required remedy or explanatory note.
    pub remedy_notes: [&'static str; 6],
}

impl FairnessReviewView {
    /// Create a 6-dimension review view with empty remedy notes.
    #[inline]
    pub const fn new(outcomes: [DimensionOutcome; 6]) -> Self {
        Self { outcomes, remedy_notes: [""; 6] }
    }

    /// Attach a required remedy note to a specific [`WellFairDimension`].
    #[inline]
    pub const fn with_remedy(mut self, dim: WellFairDimension, note: &'static str) -> Self {
        self.remedy_notes[dim as usize] = note;
        self
    }

    /// Look up the [`DimensionOutcome`] for `dim`.
    #[inline]
    pub const fn outcome(&self, dim: WellFairDimension) -> DimensionOutcome {
        self.outcomes[dim as usize]
    }

    /// Look up the remedy note for `dim`.
    #[inline]
    pub const fn remedy(&self, dim: WellFairDimension) -> &'static str {
        self.remedy_notes[dim as usize]
    }

    /// Return `true` only if every dimension is `Satisfied` or `NotApplicable`.
    #[inline]
    pub const fn all_satisfied_or_na(&self) -> bool {
        let mut i = 0;
        while i < 6 {
            if !matches!(self.outcomes[i], DimensionOutcome::Satisfied | DimensionOutcome::NotApplicable) {
                return false;
            }
            i += 1;
        }
        true
    }
}

/// Render a reactive 6-dimension WellFair Fairness & Rights Review panel.
pub fn fairness_review_panel<S, L>(review_lens: L) -> El<S>
where
    S: 'static,
    L: Lens<S, Target = FairnessReviewView>,
{
    let mut container = el("section")
        .attr("class", "wellfair-review-panel")
        .attr("data-collapsed-score", "forbidden")
        .dyn_attr("data-all-satisfied", move |ctx| ctx.get(review_lens).all_satisfied_or_na().to_string());

    for dim in WellFairDimension::ALL {
        let row = el("div")
            .attr("class", "wellfair-dimension-row")
            .attr("data-dimension", dim.code())
            .dyn_attr("data-outcome", move |ctx| ctx.get(review_lens).outcome(dim).code().to_string())
            .child(el("span").attr("class", "wellfair-dimension-title").child(text(dim.title())))
            .child(el("span").attr("class", "wellfair-dimension-badge").child(dyn_text(move |ctx| ctx.get(review_lens).outcome(dim).label().to_string())))
            .child(el("span").attr("class", "wellfair-dimension-remedy").child(dyn_text(move |ctx| ctx.get(review_lens).remedy(dim).to_string())));
        container = container.child(row);
    }
    container
}

/// Epistemic statement kind from `CONTEXTUAL-ASSERTIONS.md`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssertionKind {
    /// Direct empirical or sensor/event observation.
    Observation,
    /// Derived inference (cannot self-corroborate from a single upstream source).
    Inference,
    /// Unproven or contested allegation requiring restricted display and notice.
    Allegation,
    /// Cryptographically or institutionally signed witness attestation.
    Attestation,
    /// Formal finding resulting from an authorised resolution process.
    AdjudicatedFinding,
}

impl AssertionKind {
    /// Canonical machine code for the epistemic kind.
    #[inline]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::Inference => "inference",
            Self::Allegation => "allegation",
            Self::Attestation => "attestation",
            Self::AdjudicatedFinding => "adjudicated-finding",
        }
    }

    /// Distinct human-facing epistemic badge label.
    #[inline]
    pub const fn badge_label(&self) -> &'static str {
        match self {
            Self::Observation => "Observation (Direct Record)",
            Self::Inference => "Inference (Derived — Non-Self-Corroborating)",
            Self::Allegation => "Allegation (Unverified / Restricted Claim)",
            Self::Attestation => "Attestation (Signed Witness Statement)",
            Self::AdjudicatedFinding => "Adjudicated Finding (Due-Process Determination)",
        }
    }
}

/// Challenge and disposition status of a contextual assertion.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChallengeState {
    /// No active challenge recorded.
    Unchallenged,
    /// Formally challenged and under active review.
    ChallengedUnderReview,
    /// Restricted from downstream aggregation or public propagation.
    Restricted,
    /// Superseded by a newer attestation or adjudicated finding.
    Superseded,
}

impl ChallengeState {
    /// Canonical challenge state code.
    #[inline]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Unchallenged => "unchallenged",
            Self::ChallengedUnderReview => "challenged-under-review",
            Self::Restricted => "restricted",
            Self::Superseded => "superseded",
        }
    }

    /// Mandatory notice text accompanying the challenge state.
    #[inline]
    pub const fn notice(&self) -> &'static str {
        match self {
            Self::Unchallenged => "No active challenge",
            Self::ChallengedUnderReview => "NOTICE: Contested — Under Active Review",
            Self::Restricted => "NOTICE: Restricted — Do Not Propagate as Fact",
            Self::Superseded => "NOTICE: Superseded by Subsequent Record",
        }
    }
}

/// State model for a Contextual Assertion UI badge.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ContextualAssertionView {
    /// Epistemic class (`Observation`, `Inference`, `Allegation`, `Attestation`, `AdjudicatedFinding`).
    pub kind: AssertionKind,
    /// Challenge/restriction status.
    pub challenge: ChallengeState,
    /// Count of independent corroborating primary sources (`>= 1` required).
    pub independent_source_count: u32,
    /// Summary text of the assertion.
    pub summary: &'static str,
}

/// Render a reactive Contextual Assertion badge keeping all 5 epistemic kinds distinct.
pub fn contextual_assertion_badge<S, L>(assertion_lens: L) -> El<S>
where
    S: 'static,
    L: Lens<S, Target = ContextualAssertionView>,
{
    el("article")
        .attr("class", "contextual-assertion-badge")
        .dyn_attr("data-assertion-kind", move |ctx| ctx.get(assertion_lens).kind.code().to_string())
        .dyn_attr("data-challenge-state", move |ctx| ctx.get(assertion_lens).challenge.code().to_string())
        .dyn_attr("data-independent-sources", move |ctx| ctx.get(assertion_lens).independent_source_count.to_string())
        .child(el("span").attr("class", "assertion-kind-pill").child(dyn_text(move |ctx| ctx.get(assertion_lens).kind.badge_label().to_string())))
        .child(el("span").attr("class", "assertion-challenge-notice").child(dyn_text(move |ctx| ctx.get(assertion_lens).challenge.notice().to_string())))
        .child(el("p").attr("class", "assertion-summary").child(dyn_text(move |ctx| ctx.get(assertion_lens).summary.to_string())))
}

/// The 9 canonical stages of the Cooperative Contribution Lifecycle (`CONTRIBUTION-LIFECYCLE.md`).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u8)]
pub enum ContributionStage {
    /// Stage 1: Request & Agreement (`RequestAndAgreement`).
    RequestAndAgreement = 1,
    /// Stage 2: Activity & Resources (`ActivityAndResources`).
    ActivityAndResources = 2,
    /// Stage 3: Output or Service (`OutputOrService`).
    OutputOrService = 3,
    /// Stage 4: Submission or Claim (`SubmissionOrClaim`).
    SubmissionOrClaim = 4,
    /// Stage 5: Acceptance Decision (`AcceptanceDecision`).
    AcceptanceDecision = 5,
    /// Stage 6: Valuation (`Valuation`).
    Valuation = 6,
    /// Stage 7: Obligation (`Obligation`).
    Obligation = 7,
    /// Stage 8: Settlement Allocation (`SettlementAllocation`).
    SettlementAllocation = 8,
    /// Stage 9: Licence / Commons Transition Decision (`TransitionDecision`).
    TransitionDecision = 9,
}

impl ContributionStage {
    /// Canonical stage code.
    #[inline]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::RequestAndAgreement => "1-request-and-agreement",
            Self::ActivityAndResources => "2-activity-and-resources",
            Self::OutputOrService => "3-output-or-service",
            Self::SubmissionOrClaim => "4-submission-or-claim",
            Self::AcceptanceDecision => "5-acceptance-decision",
            Self::Valuation => "6-valuation",
            Self::Obligation => "7-obligation",
            Self::SettlementAllocation => "8-settlement-allocation",
            Self::TransitionDecision => "9-transition-decision",
        }
    }
}

/// Reactive state model for the 9-stage Contribution Lifecycle view (`Pool Funded != Contributor Paid`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ContributionStageView {
    /// Current lifecycle stage (1..=9).
    pub current_stage: ContributionStage,
    /// Total obligation threshold in integer minor units (cents).
    pub target_threshold_cents: i64,
    /// Amount accumulated in the project pool in integer minor units (cents).
    pub pool_funded_cents: i64,
    /// Amount actually allocated and settled to contributors with receipts (cents).
    pub contributor_settled_cents: i64,
    /// Explicitly waived obligation amount in integer minor units (cents).
    pub waived_obligation_cents: i64,
    /// Count of unresolved material challenges.
    pub unresolved_challenges: u32,
    /// Whether an authorised human steward has approved the transition decision.
    pub human_transition_approved: bool,
}

impl ContributionStageView {
    /// Return `true` if the pool has accumulated at least `target_threshold_cents`.
    #[inline]
    pub const fn is_pool_threshold_met(&self) -> bool {
        self.pool_funded_cents >= self.target_threshold_cents
    }

    /// Return `true` only when actual contributor settlements + waivers cover the obligation.
    #[inline]
    pub const fn is_contributor_fully_settled(&self) -> bool {
        self.contributor_settled_cents + self.waived_obligation_cents >= self.target_threshold_cents
    }

    /// Remaining unpaid gap between target obligations and settled+waived contributor receipts.
    #[inline]
    pub const fn unpaid_settled_gap_cents(&self) -> i64 {
        let covered = self.contributor_settled_cents + self.waived_obligation_cents;
        if self.target_threshold_cents > covered { self.target_threshold_cents - covered } else { 0 }
    }

    /// Return `true` only if `licence-transition-readiness` preconditions hold (`Pool Funded != Contributor Paid`).
    #[inline]
    pub const fn can_transition_to_commons(&self) -> bool {
        self.is_contributor_fully_settled()
            && self.unresolved_challenges == 0
            && self.human_transition_approved
            && matches!(self.current_stage, ContributionStage::SettlementAllocation | ContributionStage::TransitionDecision)
    }
}

/// Render a reactive 9-stage Contribution Lifecycle bar separating `Pool Funded` from `Contributor Settled`.
pub fn contribution_lifecycle_bar<S, L>(view_lens: L) -> El<S>
where
    S: 'static,
    L: Lens<S, Target = ContributionStageView>,
{
    el("section")
        .attr("class", "contribution-lifecycle-bar")
        .attr("data-pool-equals-paid", "false")
        .dyn_attr("data-stage", move |ctx| ctx.get(view_lens).current_stage.code().to_string())
        .dyn_attr("data-transition-ready", move |ctx| ctx.get(view_lens).can_transition_to_commons().to_string())
        .child(el("div").attr("class", "pool-funded-indicator").attr("data-role", "pool-funded").child(dyn_text(move |ctx| {
            let v = *ctx.get(view_lens);
            format!("Pool Funded: {} / {} cents", v.pool_funded_cents, v.target_threshold_cents)
        })))
        .child(el("div").attr("class", "contributor-settled-indicator").attr("data-role", "contributor-settled").child(dyn_text(move |ctx| {
            let v = *ctx.get(view_lens);
            format!("Contributor Settled: {} / {} cents (Unpaid Gap: {} cents)", v.contributor_settled_cents, v.target_threshold_cents, v.unpaid_settled_gap_cents())
        })))
}
