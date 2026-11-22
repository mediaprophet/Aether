// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! These unit tests use a test authority and in-memory canonical LocalHost.
//! They are not evidence of a real Sentinel or persisted QualiaDB integration.
use super::*;
use vibe::extension_registry::{
    CallExecutionContext, ExtensionRegistry, dispatch_prepared_call, prepare_extension_call,
};
use vibescript_abi::*;

#[derive(Default)]
struct TestGate {
    deny: bool,
    expired: bool,
    wrong_actor: bool,
    simulated: bool,
    fail_receipt: bool,
    receipts: usize,
    admitted: Option<AdmissionTicket>,
}
impl SentinelPolicyGate for TestGate {
    fn is_controller_alive(&self) -> bool {
        true
    }
    fn current_policy_generation(&self) -> u64 {
        1
    }
    fn admit_call(&mut self, r: &AdmissionRequest) -> Result<AdmissionTicket, VibeTrap> {
        if self.deny {
            return Err(VibeTrap::PolicyDenied);
        }
        let ticket = AdmissionTicket {
            ticket_id: 1,
            policy_generation: 1,
            actor_did_hash: if self.wrong_actor {
                r.actor_did_hash + 1
            } else {
                r.actor_did_hash
            },
            context_world_hash: r.context_world_hash,
            granted_effects: r.requested_effects,
            // Deliberately broader than ScopedGate's local bounds: authority
            // ticket identity must survive adapter admission unchanged.
            approved_envelope: ResourceEnvelope {
                max_ticks: 1_000_000,
                max_output_bytes: 8192,
                ..r.declared_envelope
            },
            expiry_lamport: if self.expired { 0 } else { u64::MAX },
            simulated: self.simulated,
        };
        self.admitted = Some(ticket);
        Ok(ticket)
    }
    fn validate_post_execution(
        &mut self,
        ticket: &AdmissionTicket,
        _: VibeOpCode,
        _: u64,
        _: usize,
        _: u64,
    ) -> Result<(), VibeTrap> {
        assert_eq!(
            Some(*ticket),
            self.admitted,
            "adapter changed authority ticket"
        );
        Ok(())
    }
    fn commit_receipt(
        &mut self,
        receipt: &ExecutionReceiptHeader,
    ) -> Result<[NQuin48; 2], VibeTrap> {
        if self.fail_receipt {
            return Err(VibeTrap::ReceiptPersistenceFailed);
        }
        self.receipts += 1;
        Ok(receipt.to_audit_nquins())
    }
}
fn context() -> CallExecutionContext {
    CallExecutionContext {
        actor_did_hash: 17,
        context_world_hash: 23,
        lamport_timestamp: 5,
        mode: ExecutionMode::GovernedApplication,
    }
}
fn runtime(limits: ExecutionLimits) -> (QualiaRuntime<vibe::LocalHost, TestGate>, UiReceiver) {
    let (tx, rx) = ui_channel(1, || {}).unwrap();
    (
        QualiaRuntime::new(
            vibe::LocalHost::default(),
            TestGate::default(),
            vec![],
            limits,
            tx,
        )
        .unwrap(),
        rx,
    )
}

#[test]
fn executes_canonical_engine_and_delivers_on_owner() {
    let (mut rt, mut ui) = runtime(ExecutionLimits::default());
    let program = rt.prepare("let answer = 6 * 7; answer;").unwrap();
    rt.execute(program, context()).unwrap();
    let mut observed = None;
    ui.drain(1, |delivery| observed = delivery.value.as_i64());
    assert_eq!(observed, Some(42));
}

#[test]
fn stale_program_cannot_execute_after_navigation() {
    let (mut rt, mut ui) = runtime(ExecutionLimits::default());
    let program = rt.prepare("1 + 2;").unwrap();
    ui.invalidate();
    assert!(matches!(
        rt.execute(program, context()),
        Err(HostError::Cancelled)
    ));
}

#[test]
fn program_cannot_cross_runtime_ownership() {
    let (a, _ui_a) = runtime(ExecutionLimits::default());
    let (mut b, _ui_b) = runtime(ExecutionLimits::default());
    let program = a.prepare("1 + 2;").unwrap();
    assert!(matches!(
        b.execute(program, context()),
        Err(HostError::WrongRuntime)
    ));
}

#[test]
fn teardown_closes_runtime_and_drops_pending_values() {
    let (rt, ui) = runtime(ExecutionLimits::default());
    drop(ui);
    assert!(matches!(rt.prepare("1;"), Err(HostError::Closed)));
}

#[test]
fn pending_result_capacity_applies_before_execution() {
    let (mut rt, _ui) = runtime(ExecutionLimits::default());
    let first = rt.prepare("1;").unwrap();
    let second = rt.prepare("2;").unwrap();
    rt.execute(first, context()).unwrap();
    assert!(matches!(
        rt.execute(second, context()),
        Err(HostError::QueueFull)
    ));
}

#[test]
fn source_and_result_limits_are_enforced() {
    let (rt, _ui) = runtime(ExecutionLimits {
        source_bytes: 2,
        ..Default::default()
    });
    assert!(matches!(
        rt.prepare("1 + 2;"),
        Err(HostError::SourceTooLarge)
    ));
    let (mut rt, mut ui) = runtime(ExecutionLimits {
        result_bytes: 1,
        ..Default::default()
    });
    let program = rt.prepare("42;").unwrap();
    assert!(
        matches!(rt.execute(program, context()), Err(HostError::Reconcile { cause, .. })
        if matches!(*cause, HostError::ResultTooLarge))
    );
    ui.drain(1, |_| panic!("oversized result published"));
}

#[test]
fn canonical_instruction_budget_is_used() {
    let (mut rt, mut ui) = runtime(ExecutionLimits {
        steps: 1,
        ..Default::default()
    });
    let program = rt.prepare("1 + 2 + 3 + 4;").unwrap();
    assert!(
        matches!(rt.execute(program, context()), Err(HostError::Reconcile { cause, .. })
        if matches!(*cause, HostError::Diagnostic(_)))
    );
    ui.drain(1, |_| panic!("budget failure published"));
}

/// Exercise the real canonical provider, descriptors, borrowed input/output and
/// dispatcher. Only Sentinel persistence is substituted by an explicitly test gate.
fn dispatch(gate: &mut TestGate) -> (Result<(), VibeTrap>, usize) {
    let registry = ExtensionRegistry::with_builtin_providers();
    let handle = prepare_extension_call(
        &registry,
        "mech",
        "kinematics",
        "kinetic_energy",
        SemVerReq::AnyStable,
        TargetProfileMask::NATIVE_DESKTOP,
    )
    .unwrap();
    let mass = 2.0_f64.to_le_bytes();
    let velocity: Vec<u8> = [3.0_f64, 4.0, 0.0]
        .into_iter()
        .flat_map(f64::to_le_bytes)
        .collect();
    let input = [
        BorrowedSliceView::from_raw_bytes(&mass, PrimitiveKind::F64, 1).unwrap(),
        BorrowedSliceView::from_raw_bytes(&velocity, PrimitiveKind::F64, 3).unwrap(),
    ];
    let mut bytes = [0; 8];
    let mut output = CallerOutputSlice::new(&mut bytes, PrimitiveKind::F64).unwrap();
    let (ui, _receiver) = ui_channel(1, || {}).unwrap();
    let epoch = ui.begin().unwrap();
    let mut scoped = crate::governance::ScopedGate {
        inner: gate,
        ui: &ui,
        epoch,
        max_receipts: 1,
        ticks_left: 100_000,
        max_output_bytes: 4096,
        receipts: vec![],
    };
    let result = dispatch_prepared_call(
        &registry,
        &handle,
        &context(),
        Some(&mut scoped),
        &input,
        &mut output,
    )
    .map(|_| ());
    (result, output.written_bytes())
}

#[test]
fn canonical_provider_receipt_is_committed() {
    let mut gate = TestGate::default();
    assert_eq!(dispatch(&mut gate), (Ok(()), 8));
    assert_eq!(gate.receipts, 1);
}

#[test]
fn denied_expired_mismatched_or_flagged_simulated_tickets_do_not_execute() {
    for (mut gate, expected) in [
        (
            TestGate {
                deny: true,
                ..Default::default()
            },
            VibeTrap::PolicyDenied,
        ),
        (
            TestGate {
                expired: true,
                ..Default::default()
            },
            VibeTrap::AuthorityExpired,
        ),
        (
            TestGate {
                wrong_actor: true,
                ..Default::default()
            },
            VibeTrap::DelegationScopeMismatch,
        ),
        (
            TestGate {
                simulated: true,
                ..Default::default()
            },
            VibeTrap::PolicyDenied,
        ),
    ] {
        assert_eq!(dispatch(&mut gate), (Err(expected), 0));
        assert_eq!(gate.receipts, 0);
    }
}

#[test]
fn receipt_failure_is_not_reported_as_commit() {
    let mut gate = TestGate {
        fail_receipt: true,
        ..Default::default()
    };
    assert_eq!(
        dispatch(&mut gate).0,
        Err(VibeTrap::ReceiptPersistenceFailed)
    );
    assert_eq!(gate.receipts, 0);
}

#[test]
fn canonical_ingress_rejects_deep_source_before_recursive_parser() {
    let (rt, _ui) = runtime(ExecutionLimits::default());
    let source = format!("{}1{};", "(".repeat(20_000), ")".repeat(20_000));
    assert!(matches!(rt.prepare(&source), Err(HostError::Diagnostic(d))
        if d.code == vibe::DiagCode::E400));
}

#[test]
fn capability_constraints_cannot_be_dropped_to_an_id_grant() {
    let (tx, _ui) = ui_channel(1, || {}).unwrap();
    let rt = QualiaRuntime::new(
        vibe::LocalHost::default(),
        TestGate::default(),
        vec!["graph.read".into()],
        ExecutionLimits::default(),
        tx,
    )
    .unwrap();
    let source = "requires [ capability(\"graph.read\", context: \"restricted\") ]; 42;";
    assert!(
        matches!(rt.prepare(source), Err(HostError::UnsupportedCapabilityConstraints(id))
        if id == "graph.read")
    );
}

#[test]
fn concrete_foundation_rejects_detached_snapshot() {
    use qualia_core_db::{
        NQuin,
        vibe_host::{PoetSnapshot, vibe_sentinel::WebizenSentinelGate},
    };
    let mut snapshot = PoetSnapshot::default();
    let mut receipts = [NQuin::default(); 2];
    let gate = WebizenSentinelGate::new(
        &[],
        &mut receipts,
        1,
        EffectSet::PURE_COMPUTE,
        ResourceEnvelope::hot_zero_heap(100_000, 4096),
    );
    let (tx, _ui) = ui_channel(1, || {}).unwrap();
    assert!(matches!(
        scoped_native_runtime(&mut snapshot, &[], gate, ExecutionLimits::default(), tx),
        Err(HostError::MissingQualiaFoundation)
    ));
}

#[test]
fn concrete_sentinel_admits_engine_operation_and_records_canonical_ram_receipt() {
    use qualia_core_db::{
        NQuin,
        modalities::logic::deontic::OP_PERMIT,
        vibe_host::{
            PoetSnapshot,
            vibe_sentinel::{WebizenSentinelGate, make_deontic_norm_quin},
        },
    };
    // Real canonical gate and host, with explicit fixture norms and caller RAM.
    // No durable database or production identity is claimed by this test.
    let mut snapshot = PoetSnapshot::from_daemon();
    let ctx = context();
    let provider = q_hash_str("https://qualiadb.org/ext/etalon/mechanics");
    let norms = [make_deontic_norm_quin(
        ctx.actor_did_hash,
        OP_PERMIT,
        false,
        provider,
        provider,
        ctx.context_world_hash,
        1000,
    )];
    let mut receipt_sink = [NQuin::default(); 2];
    let gate = WebizenSentinelGate::new(
        &norms,
        &mut receipt_sink,
        1,
        EffectSet::PURE_COMPUTE,
        ResourceEnvelope::hot_zero_heap(100_000, 4096),
    );
    let (tx, mut ui) = ui_channel(1, || {}).unwrap();
    let mut rt =
        scoped_native_runtime(&mut snapshot, &[], gate, ExecutionLimits::default(), tx).unwrap();
    let program = rt
        .prepare("mech.kinematics.kinetic_energy(2.0, [3.0, 4.0, 0.0]);")
        .unwrap();
    rt.execute(program, ctx).unwrap();
    let mut published = false;
    ui.drain(1, |delivery| {
        assert!(matches!(delivery.value, vibe::Value::F64(value) if value == 25.0));
        assert_eq!(delivery.receipts.len(), 1);
        assert!(!delivery.receipts[0].simulated);
        published = true;
    });
    assert!(published);
    let (_host, gate) = rt.into_services();
    assert_eq!(gate.persisted_receipt_quin_count(), 2);
}
