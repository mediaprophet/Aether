// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Native lifecycle adapter for the canonical, durable source-authoring service.
//! This is project source/evidence authority, not semantic graph authority.
use crate::runtime::fits_debug;
use qualia_cooperative_core::authoring::DocumentRevision;
use qualia_core_db::vibe_host::native_authority::{
    DocumentTransaction, NativeAuthority, SessionHandle, SourceReviewRequest, WorkflowEvidence,
};
use std::{fmt, marker::PhantomData, rc::Rc, sync::Arc};

#[derive(Debug)]
pub enum AuthoringBridgeError {
    InvalidLimit,
    Closed,
    SnapshotTooLarge,
    StaleScope,
    /// Canonical publication checks fail closed for stale sessions/source AND
    /// while an authority transaction holds its lock. Retry after a busy tick.
    PublicationUnavailable,
    /// Preserve the canonical service error, including reconciliation requirements.
    /// An error during a journal write must not be interpreted as proof of rollback.
    Authority(String),
}
impl fmt::Display for AuthoringBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for AuthoringBridgeError {}

/// Holds a real owner and opaque authenticated session issued by that owner.
/// It cannot create sessions, infer grants, or deserialize reusable authority.
#[derive(Clone)]
pub struct NativeAuthoringBridge {
    authority: Arc<NativeAuthority>,
    session: SessionHandle,
}

impl NativeAuthoringBridge {
    /// Revalidate the supplied native session with the actual authority service.
    pub fn connect(
        authority: Arc<NativeAuthority>,
        session: SessionHandle,
    ) -> Result<Self, AuthoringBridgeError> {
        authority
            .transact(&session, |_| DocumentTransaction {
                result: Ok(()),
                successor: None,
                reconciliation_required: false,
            })
            .map_err(AuthoringBridgeError::Authority)?;
        Ok(Self { authority, session })
    }

    /// Create on the native UI owner thread. Each view retains at most one
    /// coalesced canonical snapshot, tied to its exact source and admission generation.
    /// Call `refresh` from the application's bounded tick schedule; this is polling,
    /// not a global graph event subscription or an implicit background task.
    pub fn subscribe(
        &self,
        max_snapshot_bytes: usize,
    ) -> Result<NativeAuthoringView, AuthoringBridgeError> {
        if max_snapshot_bytes == 0 {
            return Err(AuthoringBridgeError::InvalidLimit);
        }
        let evidence = self
            .authority
            .workflow_evidence(&self.session)
            .map_err(AuthoringBridgeError::Authority)?;
        if !fits_debug(&evidence, max_snapshot_bytes) {
            return Err(AuthoringBridgeError::SnapshotTooLarge);
        }
        Ok(NativeAuthoringView {
            bridge: self.clone(),
            target: evidence.target.clone(),
            generation: evidence.authority_generation,
            max_snapshot_bytes,
            pending: Some(evidence),
            active: true,
            _owner_thread: PhantomData,
        })
    }
}

/// Native owner-scoped view. Deliberately !Send/!Sync; no DOM callback runs while
/// the canonical authority's mutex is held. Close on navigation/component teardown.
pub struct NativeAuthoringView {
    bridge: NativeAuthoringBridge,
    target: DocumentRevision,
    generation: u64,
    max_snapshot_bytes: usize,
    pending: Option<WorkflowEvidence>,
    active: bool,
    _owner_thread: PhantomData<Rc<()>>,
}

impl NativeAuthoringView {
    fn require_current(&mut self) -> Result<(), AuthoringBridgeError> {
        if !self.active {
            return Err(AuthoringBridgeError::Closed);
        }
        if !self.bridge.authority.source_publication_is_current(
            &self.bridge.session,
            self.generation,
            &self.target,
        ) {
            self.pending = None;
            return Err(AuthoringBridgeError::PublicationUnavailable);
        }
        Ok(())
    }

    /// Replace the pending snapshot rather than accumulating a queue. Canonical
    /// service/journal bounds apply before this adapter's representation bound.
    pub fn refresh(&mut self) -> Result<(), AuthoringBridgeError> {
        self.require_current()?;
        self.pending = None;
        let evidence = self
            .bridge
            .authority
            .workflow_evidence(&self.bridge.session)
            .map_err(AuthoringBridgeError::Authority)?;
        if evidence.target != self.target || evidence.authority_generation != self.generation {
            self.close();
            return Err(AuthoringBridgeError::StaleScope);
        }
        if !fits_debug(&evidence, self.max_snapshot_bytes) {
            return Err(AuthoringBridgeError::SnapshotTooLarge);
        }
        self.pending = Some(evidence);
        Ok(())
    }

    /// Return one snapshot after the canonical source-publication freshness check.
    /// The application applies it on this native thread, outside authority locks.
    /// A later revocation cannot erase values already legitimately delivered.
    pub fn take_current(&mut self) -> Result<Option<WorkflowEvidence>, AuthoringBridgeError> {
        self.require_current()?;
        Ok(self.pending.take())
    }

    /// The real canonical service validates current author/source authority and
    /// appends/syncs its journal before returning this canonical evidence record.
    /// It does not grant review authority or record an acceptance decision.
    /// Refresh separately: failure to refresh never hides a successful mutation.
    pub fn request_source_review(
        &mut self,
        id: &str,
        evidence_ref: &str,
    ) -> Result<SourceReviewRequest, AuthoringBridgeError> {
        self.require_current()?;
        self.pending = None;
        self.bridge
            .authority
            .request_source_review(&self.bridge.session, &self.target, id, evidence_ref)
            .map_err(AuthoringBridgeError::Authority)
    }

    /// Irreversibly invalidate this view on navigation/reload/component destruction.
    pub fn close(&mut self) {
        self.active = false;
        self.pending = None;
    }
}
impl Drop for NativeAuthoringView {
    fn drop(&mut self) {
        self.close();
    }
}
