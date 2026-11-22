// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Disposable test-only software-agent fixture. Never a production trust anchor.
use aether_qualia_host::{
    qualia_cooperative_core::{
        authoring::{DocumentRevision, IdentifierAssociationEvidence, PrincipalKind},
        projects::{Project, ProjectMembership, ProjectRole},
    },
    qualia_core_db::{
        crypto::network::ed25519,
        vibe_host::native_authority::{
            ADMISSION_CONTEXT, AdmissionSnapshot, NativeAuthority, SESSION_CONTEXT, SessionHandle,
            SignedAdmission, SubjectAdmission, TrustedProject, admission_digest,
            graph::{
                GRAPH_GRANT_CONTEXT, GraphGrant, GraphRights, SignedGraphGrant, graph_grant_digest,
            },
        },
    },
};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

const ISSUER_SEED: [u8; 32] = [0xa3; 32];
const AGENT_SEED: [u8; 32] = [0xb6; 32];
pub const SUBJECT: &str = "instrument:aether-disposable-window-test";
const PROJECT: &str = "aether-disposable-native-graph-test";
const IDENTIFIER: &str = "did:example:aether-native-fixture";
const ISSUER: &str = "urn:aether:test-only:issuer";
pub const CONTEXT: u64 = 99;
pub const SOURCE: &str = r#"
requires [capability("graph.read"), capability("graph.write")];
transaction {
    graph.stage(<<( <urn:fixture:subject> <urn:fixture:predicate> <urn:fixture:object> )>>);
    graph.commit();
};
graph.query(take: 8);
"#;

pub struct Fixture {
    _directory: tempfile::TempDir,
    anchor: TrustedProject,
    admission: SignedAdmission,
    document: DocumentRevision,
}
impl Fixture {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let start = now.saturating_sub(60);
        let end = now + 3600;
        let subject = SubjectAdmission {
            subject: SUBJECT.into(),
            kind: PrincipalKind::SoftwareAgent,
            controller_public_key: ed25519::public_from_seed(&AGENT_SEED),
            membership: ProjectMembership {
                id: "test-membership".into(),
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
                evidence_ref: "disposable-test-association".into(),
                valid_from_unix: start,
                valid_until_unix: end,
            }],
            reviews: vec![],
            admission_ref: "explicit-test-only-adoption".into(),
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
                name: "Disposable native window test".into(),
                description: "Test software agent; no real person or production trust claim".into(),
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
        let mut signature = [0; 64];
        ed25519::sign_with_context(
            &ISSUER_SEED,
            &admission_digest(&snapshot, 64 * 1024).unwrap(),
            ADMISSION_CONTEXT,
            &mut signature,
        )
        .unwrap();
        Self {
            _directory: tempfile::tempdir().unwrap(),
            anchor: TrustedProject {
                project_id: PROJECT.into(),
                issuer_identifier: ISSUER.into(),
                public_key: ed25519::public_from_seed(&ISSUER_SEED),
            },
            admission: SignedAdmission {
                snapshot,
                signature: signature.to_vec(),
            },
            document: DocumentRevision {
                project_id: PROJECT.into(),
                document_id: "test-native-graph".into(),
                source_id: "urn:aether:test-only:source".into(),
                revision: 1,
                source_digest: [0x64; 32],
                author_subject: Some(SUBJECT.into()),
            },
        }
    }
    pub fn open(&self) -> Arc<NativeAuthority> {
        Arc::new(
            NativeAuthority::open(
                self._directory.path(),
                self.anchor.clone(),
                self.admission.clone(),
                self.document.clone(),
                64 * 1024,
                16,
            )
            .unwrap(),
        )
    }
    pub fn grant(&self) -> SignedGraphGrant {
        let grant = GraphGrant {
            version: 1,
            project_id: PROJECT.into(),
            subject: SUBJECT.into(),
            admission_digest: admission_digest(&self.admission.snapshot, 64 * 1024).unwrap(),
            context: CONTEXT,
            rights: GraphRights {
                read: true,
                replace: true,
                subscribe: true,
            },
            max_quins: 8,
            valid_from_unix: self.admission.snapshot.valid_from_unix,
            valid_until_unix: self.admission.snapshot.valid_until_unix,
        };
        let mut signature = [0; 64];
        ed25519::sign_with_context(
            &ISSUER_SEED,
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
}
pub fn authenticate(authority: &NativeAuthority) -> SessionHandle {
    let challenge = authority.begin_session(SUBJECT, 300).unwrap();
    let mut signature = [0; 64];
    ed25519::sign_with_context(
        &AGENT_SEED,
        challenge.message_digest(),
        SESSION_CONTEXT,
        &mut signature,
    )
    .unwrap();
    authority.complete_session(challenge, &signature).unwrap()
}
