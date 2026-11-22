// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Real signed software-agent fixtures, native sessions and disk-backed canonical
//! journals. The fixed test keys do not represent enrolled people or production trust.
use crate::{AuthoringBridgeError, NativeAuthoringBridge};
use qualia_cooperative_core::{
    authoring::{DocumentRevision, IdentifierAssociationEvidence, PrincipalKind},
    projects::{Project, ProjectMembership, ProjectRole},
};
use qualia_core_db::{
    crypto::network::ed25519,
    vibe_host::native_authority::{
        ADMISSION_CONTEXT, AdmissionSnapshot, DocumentTransaction, NativeAuthority,
        SESSION_CONTEXT, SessionHandle, SignedAdmission, SubjectAdmission, TrustedProject,
        admission_digest,
    },
};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

const ISSUER_KEY: [u8; 32] = [0x96; 32];
const INSTRUMENT_KEY: [u8; 32] = [0x45; 32];
const SUBJECT: &str = "instrument:aether-authoring-test";
const PROJECT: &str = "aether-native-authoring-fixture";
const IDENTIFIER: &str = "did:example:aether-authoring-test";
const ISSUER: &str = "urn:aether:fixture:admission-authority";

struct Fixture {
    directory: tempfile::TempDir,
    anchor: TrustedProject,
    signed: SignedAdmission,
    document: DocumentRevision,
}
impl Fixture {
    fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let start = now - 60;
        let end = now + 3600;
        let subject = SubjectAdmission {
            subject: SUBJECT.into(),
            kind: PrincipalKind::SoftwareAgent,
            controller_public_key: ed25519::public_from_seed(&INSTRUMENT_KEY),
            membership: ProjectMembership {
                id: "fixture-membership".into(),
                project_id: PROJECT.into(),
                member_did: IDENTIFIER.into(),
                role: ProjectRole::Contributor,
                agreed_at_unix: u32::try_from(start).unwrap(),
                agreed_at_instant: None,
            },
            identifiers: vec![IdentifierAssociationEvidence {
                value: IDENTIFIER.into(),
                scheme: "did:example".into(),
                issuer: ISSUER.into(),
                project_id: PROJECT.into(),
                evidence_ref: "fixture-identifier-association".into(),
                valid_from_unix: start,
                valid_until_unix: end,
            }],
            reviews: vec![],
            admission_ref: "fixture-explicit-adoption".into(),
            valid_from_unix: start,
            valid_until_unix: end,
            revoked: false,
        };
        let snapshot = AdmissionSnapshot {
            version: 1,
            sequence: 1,
            issuer_identifier: ISSUER.into(),
            project: Project {
                id: PROJECT.into(),
                name: "Native source authority fixture".into(),
                description: "Software instrument test; no human enrollment claim".into(),
                licensing_ontologies: vec![],
                created_at_unix: u32::try_from(start).unwrap(),
                created_at_instant: None,
            },
            valid_from_unix: start,
            valid_until_unix: end,
            revoked: false,
            subjects: vec![subject],
            package_reviews: vec![],
        };
        let digest = admission_digest(&snapshot, 64 * 1024).unwrap();
        let mut signature = [0; 64];
        ed25519::sign_with_context(&ISSUER_KEY, &digest, ADMISSION_CONTEXT, &mut signature)
            .unwrap();
        Self {
            directory: tempfile::tempdir().unwrap(),
            anchor: TrustedProject {
                project_id: PROJECT.into(),
                issuer_identifier: ISSUER.into(),
                public_key: ed25519::public_from_seed(&ISSUER_KEY),
            },
            signed: SignedAdmission {
                snapshot,
                signature: signature.to_vec(),
            },
            document: DocumentRevision {
                project_id: PROJECT.into(),
                document_id: "native-view".into(),
                source_id: "urn:aether:fixture:source".into(),
                revision: 1,
                source_digest: [0x63; 32],
                author_subject: Some(SUBJECT.into()),
            },
        }
    }
    fn open(&self, entries: u64) -> Arc<NativeAuthority> {
        Arc::new(
            NativeAuthority::open(
                self.directory.path(),
                self.anchor.clone(),
                self.signed.clone(),
                self.document.clone(),
                64 * 1024,
                entries,
            )
            .unwrap(),
        )
    }
}

fn authenticate(authority: &NativeAuthority) -> SessionHandle {
    let challenge = authority.begin_session(SUBJECT, 300).unwrap();
    let mut signature = [0; 64];
    ed25519::sign_with_context(
        &INSTRUMENT_KEY,
        challenge.message_digest(),
        SESSION_CONTEXT,
        &mut signature,
    )
    .unwrap();
    authority.complete_session(challenge, &signature).unwrap()
}

#[test]
fn authoring_request_is_journaled_reopened_and_requires_new_proof() {
    let fixture = Fixture::new();
    let owner = fixture.open(16);
    let session = authenticate(&owner);
    let bridge = NativeAuthoringBridge::connect(owner.clone(), session.clone()).unwrap();
    let mut view = bridge.subscribe(64 * 1024).unwrap();
    assert!(view.take_current().unwrap().unwrap().requests.is_empty());
    let request = view
        .request_source_review("first-review-request", "fixture-created-source")
        .unwrap();
    assert_eq!(request.target, fixture.document);
    assert_eq!(request.requester_subject, SUBJECT);
    assert!(view.take_current().unwrap().is_none());
    view.refresh().unwrap();
    view.refresh().unwrap(); // replaces the one pending snapshot, never grows a queue
    assert_eq!(
        view.take_current().unwrap().unwrap().requests,
        vec![request.clone()]
    );
    assert!(view.take_current().unwrap().is_none());
    drop(view);
    drop(bridge);
    drop(owner);

    let reopened = fixture.open(16);
    assert!(NativeAuthoringBridge::connect(reopened.clone(), session).is_err());
    let new_session = authenticate(&reopened);
    let fresh_bridge = NativeAuthoringBridge::connect(reopened, new_session).unwrap();
    let mut restored = fresh_bridge.subscribe(64 * 1024).unwrap();
    assert_eq!(
        restored.take_current().unwrap().unwrap().requests,
        vec![request]
    );
}

#[test]
fn authoring_revocation_and_navigation_withhold_pending_evidence() {
    let fixture = Fixture::new();
    let owner = fixture.open(16);
    let session = authenticate(&owner);
    let bridge = NativeAuthoringBridge::connect(owner.clone(), session.clone()).unwrap();
    let mut view = bridge.subscribe(64 * 1024).unwrap();
    owner.disconnect(&session).unwrap();
    assert!(matches!(
        view.take_current(),
        Err(AuthoringBridgeError::PublicationUnavailable)
    ));
    assert!(matches!(
        view.request_source_review("denied", "no-session"),
        Err(AuthoringBridgeError::PublicationUnavailable)
    ));
    view.close();
    assert!(matches!(view.refresh(), Err(AuthoringBridgeError::Closed)));
}

#[test]
fn authoring_snapshot_and_actual_journal_bounds_fail_closed() {
    let fixture = Fixture::new();
    let owner = fixture.open(1); // initial signed admission consumes the only journal entry
    let session = authenticate(&owner);
    let bridge = NativeAuthoringBridge::connect(owner.clone(), session).unwrap();
    assert!(matches!(
        bridge.subscribe(1),
        Err(AuthoringBridgeError::SnapshotTooLarge)
    ));
    let mut view = bridge.subscribe(64 * 1024).unwrap();
    assert!(matches!(
        view.request_source_review("no-capacity", "bounded-fixture"),
        Err(AuthoringBridgeError::Authority(_))
    ));
    assert!(view.take_current().is_err()); // canonical owner latches reconciliation
    drop(view);
    drop(bridge);
    drop(owner);
    let reopened = fixture.open(1);
    let session = authenticate(&reopened);
    let fresh = NativeAuthoringBridge::connect(reopened, session).unwrap();
    assert!(
        fresh
            .subscribe(64 * 1024)
            .unwrap()
            .take_current()
            .unwrap()
            .unwrap()
            .requests
            .is_empty()
    );
}

#[test]
fn authoring_busy_owner_withholds_snapshot_but_can_retry_after_transaction() {
    let fixture = Fixture::new();
    let owner = fixture.open(16);
    let session = authenticate(&owner);
    let bridge = NativeAuthoringBridge::connect(owner.clone(), session.clone()).unwrap();
    let mut view = bridge.subscribe(64 * 1024).unwrap();
    let (locked_tx, locked_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        owner
            .transact(&session, |_| {
                locked_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                DocumentTransaction {
                    result: Ok(()),
                    successor: None,
                    reconciliation_required: false,
                }
            })
            .unwrap()
    });
    locked_rx.recv().unwrap();
    assert!(matches!(
        view.take_current(),
        Err(AuthoringBridgeError::PublicationUnavailable)
    ));
    release_tx.send(()).unwrap();
    worker.join().unwrap();
    view.refresh().unwrap();
    assert!(view.take_current().unwrap().is_some());
}

fn graph_grant(
    fixture: &Fixture,
    context: u64,
) -> qualia_core_db::vibe_host::native_authority::graph::SignedGraphGrant {
    use qualia_core_db::vibe_host::native_authority::graph::*;
    let grant = GraphGrant {
        version: 1,
        project_id: PROJECT.into(),
        subject: SUBJECT.into(),
        admission_digest: admission_digest(&fixture.signed.snapshot, 64 * 1024).unwrap(),
        context,
        rights: GraphRights {
            read: true,
            replace: true,
            subscribe: true,
        },
        max_quins: 8,
        valid_from_unix: fixture.signed.snapshot.valid_from_unix,
        valid_until_unix: fixture.signed.snapshot.valid_until_unix,
    };
    let mut signature = [0; 64];
    ed25519::sign_with_context(
        &ISSUER_KEY,
        &graph_grant_digest(&grant).unwrap(),
        GRAPH_GRANT_CONTEXT,
        &mut signature,
    )
    .unwrap();
    SignedGraphGrant {
        grant,
        signature: signature.to_vec(),
    }
}

fn chora_context() -> (
    crate::chora_protocol::RouteStamp,
    crate::chora_protocol::CallerContext,
) {
    use crate::chora_protocol::*;
    (
        RouteStamp {
            session: [4; 16],
            view: 1,
            generation: 2,
            document: 3,
            viewport: 4,
        },
        CallerContext {
            engine_instance: [5; 16],
            webview: "actual-view".into(),
            pipeline: "actual-pipeline".into(),
            navigation_epoch: [6; 16],
            origin: "https://example.test".into(),
            private_browsing: false,
        },
    )
}

#[test]
fn chora_parent_reads_real_signed_graph_and_rechecks_revocation() {
    use crate::{ChoraGraphRoutes, NativeGraphBridge, chora_protocol::*};
    use qualia_core_db::NQuin;
    use std::{sync::atomic::AtomicBool, time::Instant};
    let fixture = Fixture::new();
    let owner = fixture.open(16);
    let session = authenticate(&owner);
    let grant = graph_grant(&fixture, 99);
    let bridge = NativeGraphBridge::connect(owner.clone(), &session, &grant).unwrap();
    let quin = NQuin {
        subject: u64::MAX,
        predicate: 2,
        object: 3,
        context: 99,
        metadata: u64::MAX,
        parity: NQuin::calculate_parity(u64::MAX, 2, 3, 99, u64::MAX),
    };
    bridge.replace(0, &[quin], &AtomicBool::new(false)).unwrap();
    let (stamp, caller) = chora_context();
    let mut routes = ChoraGraphRoutes::new();
    let request = |id, caller| ControlEnvelope {
        stamp,
        sequence: 1,
        message: Control::PageRequest {
            id,
            caller,
            channel: "qualia.graph.snapshot".into(),
            payload: "{}".into(),
        },
    };
    assert_eq!(
        routes.accept(&request([1; 16], caller.clone()), Instant::now()),
        Err(ReplyError::Denied)
    );
    routes
        .register(
            stamp,
            caller.clone(),
            ExecutionProfile::TrustedEmbedding,
            bridge,
        )
        .unwrap();
    for field in 0..5 {
        let mut changed = stamp;
        match field {
            0 => changed.session[0] ^= 1,
            1 => changed.view += 1,
            2 => changed.generation += 1,
            3 => changed.document += 1,
            _ => changed.viewport += 1,
        }
        let mut stale = request([8; 16], caller.clone());
        stale.stamp = changed;
        assert_eq!(
            routes.accept(&stale, Instant::now()),
            Err(ReplyError::Denied)
        );
    }
    for channel in ["qualia.graph.replace", "vibe.execute", "host.keystore"] {
        let mut denied = request([8; 16], caller.clone());
        if let Control::PageRequest { channel: route, .. } = &mut denied.message {
            *route = channel.into();
        }
        assert_eq!(
            routes.accept(&denied, Instant::now()),
            Err(ReplyError::Denied)
        );
    }
    let mut forged = caller.clone();
    forged.origin = "https://attacker.test".into();
    assert_eq!(
        routes.accept(&request([1; 16], forged), Instant::now()),
        Err(ReplyError::Denied)
    );
    let mut private = caller.clone();
    private.private_browsing = true;
    assert_eq!(
        routes.accept(&request([1; 16], private), Instant::now()),
        Err(ReplyError::Denied)
    );
    routes
        .accept(&request([1; 16], caller.clone()), Instant::now())
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&routes.finish(stamp, [1; 16], Instant::now()).unwrap()).unwrap();
    assert_eq!(json["context"], "99");
    assert_eq!(json["revision"], "1");
    let expected = [
        quin.subject,
        quin.predicate,
        quin.object,
        quin.context,
        quin.metadata,
        quin.parity,
    ]
    .map(|n| n.to_string());
    for (lane, value) in expected.into_iter().enumerate() {
        assert_eq!(json["quins"][0][lane], value);
    }
    assert_eq!(
        routes.finish(stamp, [1; 16], Instant::now()),
        Err(ReplyError::Cancelled)
    );
    routes
        .accept(&request([2; 16], caller), Instant::now())
        .unwrap();
    owner.disconnect(&session).unwrap();
    assert_eq!(
        routes.finish(stamp, [2; 16], Instant::now()),
        Err(ReplyError::Denied)
    );
}

#[test]
fn chora_parent_bounds_cancels_expires_and_invalidates_exact_requests() {
    use crate::{ChoraGraphRoutes, NativeGraphBridge, chora_protocol::*};
    use std::time::{Duration, Instant};
    let fixture = Fixture::new();
    let owner = fixture.open(16);
    let session = authenticate(&owner);
    let bridge =
        NativeGraphBridge::connect(owner.clone(), &session, &graph_grant(&fixture, 99)).unwrap();
    let (stamp, caller) = chora_context();
    let mut routes = ChoraGraphRoutes::new();
    routes
        .register(
            stamp,
            caller.clone(),
            ExecutionProfile::TrustedEmbedding,
            bridge,
        )
        .unwrap();
    let request = |id| ControlEnvelope {
        stamp,
        sequence: 1,
        message: Control::PageRequest {
            id,
            caller: caller.clone(),
            channel: "qualia.graph.snapshot".into(),
            payload: "{}".into(),
        },
    };
    let now = Instant::now();
    for i in 0..MAX_PENDING {
        routes.accept(&request([i as u8; 16]), now).unwrap();
    }
    assert_eq!(
        routes.accept(&request([99; 16]), now),
        Err(ReplyError::Busy)
    );
    assert_eq!(
        routes.accept(&request([0; 16]), now),
        Err(ReplyError::InvalidContext)
    );
    routes.cancel(stamp, [0; 16]);
    assert_eq!(
        routes.finish(stamp, [0; 16], now),
        Err(ReplyError::Cancelled)
    );
    routes.accept(&request([99; 16]), now).unwrap();
    assert_eq!(
        routes
            .expire(now + Duration::from_secs(MAX_DEADLINE_SECS))
            .len(),
        MAX_PENDING
    );
    routes.accept(&request([1; 16]), now).unwrap();
    assert_eq!(
        routes.finish(stamp, [1; 16], now + Duration::from_secs(MAX_DEADLINE_SECS)),
        Err(ReplyError::TimedOut)
    );
    routes.accept(&request([2; 16]), now).unwrap();
    let mut resized = stamp;
    resized.viewport += 1;
    routes.invalidate(resized);
    assert_eq!(
        routes.finish(stamp, [2; 16], now),
        Err(ReplyError::Cancelled)
    );
    assert_eq!(
        routes.accept(&request([3; 16]), now),
        Err(ReplyError::Denied)
    );
    #[cfg(feature = "chora-client")]
    {
        let fresh =
            NativeGraphBridge::connect(owner, &session, &graph_grant(&fixture, 99)).unwrap();
        routes
            .register(
                resized,
                caller.clone(),
                ExecutionProfile::TrustedEmbedding,
                fresh,
            )
            .unwrap();
        let mut request = request([5; 16]);
        request.stamp = resized;
        routes.accept(&request, now).unwrap();
        // An already queued acknowledgement of this same resize must preserve
        // the explicitly registered new owner and its retained real request.
        routes.advance_view(resized);
        assert!(routes.finish(resized, [5; 16], now).is_ok());
    }
    routes.close();
    assert_eq!(
        routes.accept(&request([4; 16]), now),
        Err(ReplyError::Cancelled)
    );
}
#[test]
fn canonical_graph_view_coalesces_and_withholds_revoked_data() {
    use crate::NativeGraphBridge;
    use qualia_core_db::{NQuin, vibe_host::native_authority::graph::GraphError};
    use std::sync::atomic::AtomicBool;
    let fixture = Fixture::new();
    let owner = fixture.open(16);
    let session = authenticate(&owner);
    let grant = graph_grant(&fixture, 99);
    let writer = NativeGraphBridge::connect(owner.clone(), &session, &grant).unwrap();
    let mut view = NativeGraphBridge::connect(owner.clone(), &session, &grant)
        .unwrap()
        .subscribe()
        .unwrap();
    assert_eq!(view.poll().unwrap().unwrap().read.revision, 0);
    let mut quin = NQuin {
        subject: 1,
        predicate: 2,
        object: 3,
        context: 99,
        metadata: u64::MAX,
        parity: NQuin::calculate_parity(1, 2, 3, 99, u64::MAX),
    };
    writer.replace(0, &[quin], &AtomicBool::new(false)).unwrap();
    quin.object = 4;
    quin.recalculate_parity();
    writer.replace(1, &[quin], &AtomicBool::new(false)).unwrap();
    let snapshot = view.poll().unwrap().unwrap();
    assert_eq!(snapshot.read.revision, 2);
    assert_eq!(snapshot.skipped_revisions, 1);
    assert_eq!(snapshot.quins, &[quin]);
    assert!(view.poll().unwrap().is_none());
    owner.disconnect(&session).unwrap();
    assert!(matches!(view.poll(), Err(GraphError::Authentication)));
    view.close();
    assert!(matches!(view.poll(), Err(GraphError::Cancelled)));
}
#[test]
fn graph_bridge_rejects_forged_scope_and_retains_committed_revision_on_reopen() {
    use crate::NativeGraphBridge;
    use qualia_core_db::{NQuin, vibe_host::native_authority::graph::GraphError};
    use std::sync::atomic::AtomicBool;
    let fixture = Fixture::new();
    let owner = fixture.open(16);
    let session = authenticate(&owner);
    let grant = graph_grant(&fixture, 99);
    let mut forged = grant.clone();
    forged.grant.context = 100;
    assert!(matches!(
        NativeGraphBridge::connect(owner.clone(), &session, &forged),
        Err(GraphError::Signature)
    ));
    let writer = NativeGraphBridge::connect(owner.clone(), &session, &grant).unwrap();
    let quin = NQuin {
        subject: 1,
        predicate: 2,
        object: 3,
        context: 99,
        metadata: 4,
        parity: NQuin::calculate_parity(1, 2, 3, 99, 4),
    };
    assert_eq!(
        writer.replace(0, &[quin], &AtomicBool::new(true)),
        Err(GraphError::Cancelled)
    );
    writer.replace(0, &[quin], &AtomicBool::new(false)).unwrap();
    assert_eq!(
        writer.replace(0, &[], &AtomicBool::new(false)),
        Err(GraphError::RevisionConflict)
    );
    drop(writer);
    drop(owner);
    let owner = fixture.open(16);
    assert!(NativeGraphBridge::connect(owner.clone(), &session, &grant).is_err());
    let session = authenticate(&owner);
    let mut view = NativeGraphBridge::connect(owner, &session, &grant)
        .unwrap()
        .subscribe()
        .unwrap();
    let snapshot = view.poll().unwrap().unwrap();
    assert_eq!(snapshot.read.revision, 1);
    assert_eq!(snapshot.quins, &[quin]);
}

#[test]
fn signed_graph_vibe_transaction_persists_and_delivers_canonical_values_to_ui() {
    use crate::{ExecutionLimits, QualiaRuntime, ui_channel};
    use qualia_core_db::{
        NQuin,
        vibe_host::{native_authority::graph::NativeGraphHost, vibe_sentinel::WebizenSentinelGate},
    };
    use std::sync::atomic::{AtomicBool, Ordering};
    use vibe::{Host, Span, Value, extension_registry::CallExecutionContext};
    use vibescript_abi::{EffectSet, ExecutionMode, ResourceEnvelope};
    let fixture = Fixture::new();
    let owner = fixture.open(16);
    let session = authenticate(&owner);
    let grant = graph_grant(&fixture, 99);
    let scope = owner.bind_graph_scope(&session, &grant).unwrap();
    let cancelled = Arc::new(AtomicBool::new(false));
    let host = NativeGraphHost::new(owner.clone(), scope, cancelled.clone()).unwrap();
    let native_provider = vibescript_abi::q_hash_str("vibe:native");
    let norms = [
        qualia_core_db::vibe_host::vibe_sentinel::make_deontic_norm_quin(
            vibescript_abi::q_hash_str(SUBJECT),
            qualia_core_db::modalities::logic::deontic::OP_PERMIT,
            false,
            native_provider,
            native_provider,
            99,
            1000,
        ),
    ];
    let mut audit = [NQuin::default(); 6];
    let gate = WebizenSentinelGate::new(
        &norms,
        &mut audit,
        1,
        EffectSet::PURE_COMPUTE
            .union(EffectSet::GRAPH_READ)
            .union(EffectSet::GRAPH_MUTATE),
        ResourceEnvelope::hot_zero_heap(100_000, 4096),
    );
    let (sender, mut receiver) = ui_channel(1, || {}).unwrap();
    let mut runtime = QualiaRuntime::new(
        host,
        gate,
        vec!["graph.read".into(), "graph.write".into()],
        ExecutionLimits::default(),
        sender,
    )
    .unwrap();
    let source = r#"
requires [capability("graph.read"), capability("graph.write")];
transaction {
    graph.stage(<<( <urn:aether:subject> <urn:aether:predicate> <urn:aether:object> )>>);
    graph.commit();
};
graph.query(take: 8);
"#;
    let program = runtime.prepare(source).unwrap();
    runtime
        .execute(
            program,
            CallExecutionContext {
                actor_did_hash: vibescript_abi::q_hash_str(SUBJECT),
                context_world_hash: 99,
                lamport_timestamp: 1,
                mode: ExecutionMode::GovernedApplication,
            },
        )
        .unwrap();
    let mut delivered = Vec::new();
    receiver.drain(1, |delivery| {
        let Value::List(values) = delivery.value else {
            panic!("expected canonical graph list")
        };
        delivered = values;
        assert_eq!(delivery.receipts.len(), 3);
    });
    assert_eq!(delivered.len(), 1);
    let Value::QuinRef(quin) = &delivered[0] else {
        panic!("expected canonical QuinRef")
    };
    assert_eq!(quin.raw_fields()[3], 99);
    let (mut host, _) = runtime.into_services();
    assert_eq!(host.last_receipt().unwrap().revision, 1);
    assert!(
        matches!(host.graph_query(&[], 0, Span::point(0)).unwrap(), Value::List(rows) if rows.is_empty())
    );
    cancelled.store(true, Ordering::Release);
    assert!(host.graph_query(&[], 8, Span::point(0)).is_err());
    drop(host);
    drop(owner);
    let reopened = fixture.open(16);
    let session = authenticate(&reopened);
    let scope = reopened.bind_graph_scope(&session, &grant).unwrap();
    let mut rows = [NQuin::default(); 8];
    assert_eq!(reopened.read_graph(&scope, &mut rows).unwrap().revision, 1);
    assert_eq!(
        quin.raw_fields(),
        [
            rows[0].subject,
            rows[0].predicate,
            rows[0].object,
            rows[0].context,
            rows[0].metadata,
            rows[0].parity
        ]
    );
}

#[test]
fn native_graph_runtime_reserves_host_workspace_before_construction() {
    use crate::{ExecutionLimits, HostError, native_graph_runtime, ui_channel};
    use qualia_core_db::{
        NQuin,
        vibe_host::{
            native_authority::graph::{GraphError, NativeGraphHost},
            vibe_sentinel::WebizenSentinelGate,
        },
    };
    use std::sync::atomic::AtomicBool;
    use vibescript_abi::{EffectSet, ResourceEnvelope};
    let fixture = Fixture::new();
    let owner = fixture.open(16);
    let session = authenticate(&owner);
    let grant = graph_grant(&fixture, 99);
    let mut audit = [NQuin::default(); 2];
    let gate = WebizenSentinelGate::new(
        &[],
        &mut audit,
        1,
        EffectSet::PURE_COMPUTE,
        ResourceEnvelope::hot_zero_heap(100_000, 4096),
    );
    let (sender, _receiver) = ui_channel(1, || {}).unwrap();
    let result = native_graph_runtime(
        owner.clone(),
        &session,
        &grant,
        gate,
        ExecutionLimits {
            workspace_bytes: 1,
            ..Default::default()
        },
        sender,
        Arc::new(AtomicBool::new(false)),
    );
    assert!(matches!(
        result,
        Err(HostError::Graph(GraphError::Capacity))
    ));

    let scope = owner.bind_graph_scope(&session, &grant).unwrap();
    let reservation =
        256 * 1024 + 2 * scope.max_quins() as u64 * std::mem::size_of::<NQuin>() as u64;
    let mut budget = vibe::Budget {
        steps_left: 10,
        workspace_left: reservation + 123,
    };
    let _host = NativeGraphHost::new_with_budget(
        owner,
        scope,
        Arc::new(AtomicBool::new(false)),
        &mut budget,
    )
    .unwrap();
    assert_eq!(budget.workspace_left, 123);
    assert_eq!(budget.steps_left, 10);
}

#[test]
fn graph_component_task_delivers_on_owner_and_cancels_worker_on_dom_drop() {
    use crate::{NativeGraphBridge, NativeGraphView, native_ui::use_qualia_graph};
    use aether_core::{Element, ScopeId, Task, VNode, VirtualDom};
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
        sync::atomic::{AtomicBool, Ordering},
    };
    #[derive(Clone)]
    struct Props {
        view: Rc<RefCell<Option<NativeGraphView>>>,
        cancelled: Arc<AtomicBool>,
        task: Rc<Cell<Option<Task>>>,
        deliveries: Rc<Cell<usize>>,
        cancelled_events: Rc<Cell<usize>>,
        owner: std::thread::ThreadId,
        tick_open: bool,
    }
    fn app(props: Props) -> Element {
        let task = use_qualia_graph(
            move || props.view.borrow_mut().take().unwrap(),
            props.cancelled,
            move || std::future::ready(props.tick_open),
            move |result| {
                assert_eq!(std::thread::current().id(), props.owner);
                if matches!(
                    &result,
                    Err(qualia_core_db::vibe_host::native_authority::graph::GraphError::Cancelled)
                ) {
                    props.cancelled_events.set(props.cancelled_events.get() + 1);
                    return;
                }
                if let Some(snapshot) = result.unwrap() {
                    assert_eq!(snapshot.read.context, 99);
                    props.deliveries.set(props.deliveries.get() + 1);
                }
            },
        );
        props.task.set(Some(task));
        VNode::empty()
    }
    for (poll, tick_open, external_cancel) in [
        (false, true, false),
        (true, true, false),
        (true, false, false),
        (true, true, true),
    ] {
        let fixture = Fixture::new();
        let owner = fixture.open(16);
        let session = authenticate(&owner);
        let grant = graph_grant(&fixture, 99);
        let view = NativeGraphBridge::connect(owner, &session, &grant)
            .unwrap()
            .subscribe()
            .unwrap();
        let props = Props {
            view: Rc::new(RefCell::new(Some(view))),
            cancelled: Arc::new(AtomicBool::new(false)),
            task: Rc::new(Cell::new(None)),
            deliveries: Rc::new(Cell::new(0)),
            cancelled_events: Rc::new(Cell::new(0)),
            owner: std::thread::current().id(),
            tick_open,
        };
        let mut dom = VirtualDom::new_with_props(app, props.clone());
        dom.rebuild_in_place();
        if poll {
            assert!(
                dom.in_scope(ScopeId::APP, || props.task.get().unwrap().poll_now())
                    .is_pending()
                    == tick_open
            );
            assert_eq!(props.deliveries.get(), usize::from(tick_open));
        }
        if external_cancel {
            props.cancelled.store(true, Ordering::Release);
            assert!(
                dom.in_scope(ScopeId::APP, || props.task.get().unwrap().poll_now())
                    .is_ready()
            );
            assert_eq!(props.deliveries.get(), 1);
        }
        let closed = external_cancel || (poll && !tick_open);
        assert_eq!(props.cancelled_events.get(), usize::from(closed));
        assert_eq!(props.cancelled.load(Ordering::Acquire), closed);
        drop(dom);
        assert!(props.cancelled.load(Ordering::Acquire));
    }
}

#[test]
fn actual_bytecode_cannot_stage_without_native_transaction_admission() {
    use qualia_core_db::{
        NQuin,
        vibe_host::{native_authority::graph::NativeGraphHost, vibe_sentinel::WebizenSentinelGate},
    };
    use std::sync::atomic::AtomicBool;
    use vibe::{
        Budget, QuinRef, Value,
        bytecode::{Vm, compile},
        extension_registry::CallExecutionContext,
    };
    use vibescript_abi::{EffectSet, ExecutionMode, ResourceEnvelope};
    let fixture = Fixture::new();
    let owner = fixture.open(16);
    let session = authenticate(&owner);
    let grant = graph_grant(&fixture, 99);
    let scope = owner.bind_graph_scope(&session, &grant).unwrap();
    let mut host =
        NativeGraphHost::new(owner.clone(), scope, Arc::new(AtomicBool::new(false))).unwrap();
    // The real compiler currently emits this transaction as a plain block. Even
    // with a genuine signed scope and live Sentinel, the host must require begin.
    let program = vibe::parse_program(
        "fn submit(q: Quin) { transaction { graph.stage(q); graph.commit(); } }",
    )
    .unwrap();
    let chunk = compile(&program).unwrap();
    let native_provider = vibescript_abi::q_hash_str("vibe:native");
    let norms = [
        qualia_core_db::vibe_host::vibe_sentinel::make_deontic_norm_quin(
            vibescript_abi::q_hash_str(SUBJECT),
            qualia_core_db::modalities::logic::deontic::OP_PERMIT,
            false,
            native_provider,
            native_provider,
            99,
            1000,
        ),
    ];
    let mut audit = [NQuin::default(); 6];
    let mut gate = WebizenSentinelGate::new(
        &norms,
        &mut audit,
        1,
        EffectSet::PURE_COMPUTE
            .union(EffectSet::GRAPH_READ)
            .union(EffectSet::GRAPH_MUTATE),
        ResourceEnvelope::hot_zero_heap(100_000, 4096),
    );
    let mut vm = Vm::new(&chunk, &mut host, Budget::default()).with_extension_governance(
        CallExecutionContext {
            actor_did_hash: vibescript_abi::q_hash_str(SUBJECT),
            context_world_hash: 99,
            lamport_timestamp: 1,
            mode: ExecutionMode::GovernedApplication,
        },
        Some(&mut gate),
    );
    vm.run().unwrap();
    let term = Value::QuinRef(QuinRef::from_raw(
        1,
        2,
        3,
        99,
        4,
        NQuin::calculate_parity(1, 2, 3, 99, 4),
    ));
    let error = vm
        .call_function(chunk.find_function("submit").unwrap(), &[term])
        .unwrap_err();
    assert!(
        error.to_string().contains("native graph: Permission"),
        "{error}"
    );
    drop(vm);
    assert!(host.last_receipt().is_none());
    let scope = owner.bind_graph_scope(&session, &grant).unwrap();
    let mut rows = [NQuin::default(); 8];
    let read = owner.read_graph(&scope, &mut rows).unwrap();
    assert_eq!((read.revision, read.written), (0, 0));
}

#[test]
fn declarative_graph_and_authoring_hooks_release_views_on_unmount() {
    use crate::hooks::{AuthoringBridge, use_authoring_bridge, use_qualia_subscription};
    use aether_core::{Element, VNode, VirtualDom};
    use std::{
        cell::RefCell,
        future::pending,
        rc::Rc,
        sync::atomic::{AtomicBool, Ordering},
    };

    #[derive(Clone)]
    struct Props {
        graph: Rc<RefCell<Option<crate::NativeGraphView>>>,
        authoring: Rc<RefCell<Option<crate::NativeAuthoringView>>>,
        cancellation: Arc<AtomicBool>,
        handle: Rc<RefCell<Option<AuthoringBridge>>>,
    }
    fn root(props: Props) -> Element {
        use_qualia_subscription(
            || props.graph.borrow_mut().take().unwrap(),
            props.cancellation,
            pending::<bool>,
            |snapshot| snapshot.quins.len(),
        );
        let handle = use_authoring_bridge(
            || props.authoring.borrow_mut().take().unwrap(),
            pending::<bool>,
        );
        *props.handle.borrow_mut() = Some(handle);
        VNode::empty()
    }
    let fixture = Fixture::new();
    let authority = fixture.open(8);
    let session = authenticate(&authority);
    let graph =
        crate::NativeGraphBridge::connect(authority.clone(), &session, &graph_grant(&fixture, 6))
            .unwrap()
            .subscribe()
            .unwrap();
    let authoring = NativeAuthoringBridge::connect(authority, session)
        .unwrap()
        .subscribe(64 * 1024)
        .unwrap();
    let props = Props {
        graph: Rc::new(RefCell::new(Some(graph))),
        authoring: Rc::new(RefCell::new(Some(authoring))),
        cancellation: Arc::new(AtomicBool::new(false)),
        handle: Rc::new(RefCell::new(None)),
    };
    let mut dom = VirtualDom::new_with_props(root, props.clone());
    dom.rebuild_in_place();
    assert!(!props.cancellation.load(Ordering::Acquire));
    let handle = props.handle.borrow_mut().take().unwrap();
    drop(dom);
    assert!(props.cancellation.load(Ordering::Acquire));
    assert!(matches!(
        handle.request_source_review("after-unmount", "evidence"),
        Err(AuthoringBridgeError::Closed)
    ));
}
