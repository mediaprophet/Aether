// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Canonical project-graph access on a native owner, separate from daemon globals.
use qualia_core_db::{
    NQuin,
    vibe_host::native_authority::{
        NativeAuthority, SessionHandle,
        graph::{
            GraphError, GraphRead, GraphReceipt, GraphScope, GraphSubscription, SignedGraphGrant,
        },
    },
};
use std::{
    marker::PhantomData,
    rc::Rc,
    sync::{Arc, atomic::AtomicBool},
};

/// Holds only a genuine canonical scope; it cannot synthesize graph authority from
/// a source-authoring session, a page message, a context hash or a capability name.
pub struct NativeGraphBridge {
    authority: Arc<NativeAuthority>,
    scope: GraphScope,
}
impl NativeGraphBridge {
    /// Fixed scratch capacity required by this canonical exact-context scope.
    pub fn snapshot_capacity(&self) -> usize {
        self.scope.max_quins()
    }

    /// Revalidates signed scope/session on every nonblocking read. The caller
    /// owns and clears scratch; no authority is inferred from transport data.
    pub fn read_snapshot(&self, output: &mut [NQuin]) -> Result<GraphRead, GraphError> {
        self.authority.read_graph(&self.scope, output)
    }

    pub fn connect(
        authority: Arc<NativeAuthority>,
        session: &SessionHandle,
        grant: &SignedGraphGrant,
    ) -> Result<Self, GraphError> {
        let scope = authority.bind_graph_scope(session, grant)?;
        Ok(Self { authority, scope })
    }

    /// Cold journalled authoring: call on an application-owned worker, never from
    /// a render callback. Once journal append begins cancellation cannot undo it.
    /// A successful receipt must be retained even if the UI owner has since closed.
    pub fn replace(
        &self,
        expected_revision: u64,
        quins: &[NQuin],
        cancelled: &AtomicBool,
    ) -> Result<GraphReceipt, GraphError> {
        self.authority
            .replace_graph(&self.scope, expected_revision, quins, cancelled)
    }

    /// Construct on the UI owner. One bounded snapshot buffer, no growing event
    /// queue or ambient/global revision subscription. Poll in the native tick.
    pub fn subscribe(self) -> Result<NativeGraphView, GraphError> {
        let capacity = self.scope.max_quins();
        let subscription = self.authority.subscribe_graph(self.scope)?;
        let mut buffer = Vec::new();
        buffer
            .try_reserve_exact(capacity)
            .map_err(|_| GraphError::Capacity)?;
        buffer.resize(capacity, NQuin::default());
        Ok(NativeGraphView {
            authority: self.authority,
            subscription,
            buffer,
            last_revision: None,
            _owner_thread: PhantomData,
        })
    }
}

pub struct GraphSnapshot<'a> {
    pub read: GraphRead,
    pub quins: &'a [NQuin],
    /// Revisions coalesced since this view's previous delivered snapshot. The
    /// complete current snapshot replaces them; this is not a stream of deltas.
    pub skipped_revisions: u64,
}

/// !Send/!Sync ensures view application occurs on its native owner thread.
/// No user callback runs under the canonical authority's lock. Closing or dropping
/// the view ends its local subscription; it does not revoke other granted clients.
pub struct NativeGraphView {
    authority: Arc<NativeAuthority>,
    subscription: GraphSubscription,
    buffer: Vec<NQuin>,
    last_revision: Option<u64>,
    _owner_thread: PhantomData<Rc<()>>,
}
impl NativeGraphView {
    /// Revalidate on every poll, even with no updates. Returned canonical lanes
    /// borrow this view and cannot survive its teardown without an explicit copy.
    /// Callers must clear displayed private data on revocation/closed errors.
    pub fn poll(&mut self) -> Result<Option<GraphSnapshot<'_>>, GraphError> {
        let result = self
            .authority
            .poll_graph(&mut self.subscription, &mut self.buffer);
        match result {
            Ok(Some(read)) => {
                let skipped_revisions = self.last_revision.map_or(0, |previous| {
                    read.revision.saturating_sub(previous).saturating_sub(1)
                });
                self.last_revision = Some(read.revision);
                Ok(Some(GraphSnapshot {
                    read,
                    quins: &self.buffer[..read.written],
                    skipped_revisions,
                }))
            }
            Ok(None) => Ok(None),
            Err(GraphError::Busy) => Err(GraphError::Busy),
            Err(error) => {
                self.buffer.fill(NQuin::default());
                Err(error)
            }
        }
    }
    pub fn close(&mut self) {
        self.subscription.close();
        self.buffer.fill(NQuin::default());
    }
}
impl Drop for NativeGraphView {
    fn drop(&mut self) {
        self.close();
    }
}

/// Concrete canonical runtime for the authority-owned project graph profile.
/// The supplied Sentinel governs extensions; graph effects use the separate
/// signed NativeAuthority scope and durable journal, never inferred `requires`.
pub type NativeGraphRuntime<'policy> = crate::QualiaRuntime<
    qualia_core_db::vibe_host::native_authority::graph::NativeGraphHost,
    qualia_core_db::vibe_host::vibe_sentinel::WebizenSentinelGate<'policy>,
>;

/// Compose already-adopted authority and native session with explicit graph rights.
/// Construct on an application worker; cancel its token on navigation/teardown.
/// A parser, provider, or OS process isolation boundary is not created here.
#[allow(clippy::too_many_arguments)]
pub fn native_graph_runtime<'policy>(
    authority: Arc<NativeAuthority>,
    session: &SessionHandle,
    grant: &SignedGraphGrant,
    gate: qualia_core_db::vibe_host::vibe_sentinel::WebizenSentinelGate<'policy>,
    mut limits: crate::ExecutionLimits,
    ui: crate::UiSender,
    cancelled: Arc<AtomicBool>,
) -> Result<NativeGraphRuntime<'policy>, crate::HostError> {
    use qualia_core_db::vibe_host::native_authority::graph::NativeGraphHost;
    let scope = authority
        .bind_graph_scope(session, grant)
        .map_err(crate::HostError::Graph)?;
    let mut capabilities = Vec::new();
    if grant.grant.rights.read {
        capabilities.push("graph.read".to_owned());
    }
    if grant.grant.rights.read && grant.grant.rights.replace {
        capabilities.push("graph.write".to_owned());
    }
    let mut budget = vibe::Budget {
        steps_left: limits.steps,
        workspace_left: limits.workspace_bytes,
    };
    let host = NativeGraphHost::new_with_budget(authority, scope, cancelled, &mut budget)
        .map_err(crate::HostError::Graph)?;
    limits.workspace_bytes = budget.workspace_left;
    crate::QualiaRuntime::new(host, gate, capabilities, limits, ui)
}
