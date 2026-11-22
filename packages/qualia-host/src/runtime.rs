// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
use crate::{UiDelivery, UiSender, governance::ScopedGate};
use std::{
    fmt::{self, Write},
    sync::Arc,
};
use vibe::extension_registry::CallExecutionContext;
use vibe::{Budget, Engine, Env, Host, Program};
use vibescript_abi::{ExecutionMode, ExecutionReceiptHeader, SentinelPolicyGate, VibeTrap};

/// Cold-path and interpreter bounds, not a hard allocator or wall-clock quota.
#[derive(Clone, Debug)]
pub struct ExecutionLimits {
    pub source_bytes: usize,
    pub steps: u64,
    pub workspace_bytes: u64,
    /// Maximum Debug representation of a delivered canonical value. Measured
    /// without allocating a serialization buffer; not an ABI or heap-size claim.
    pub result_bytes: usize,
    pub extension_output_bytes: usize,
    pub receipts: usize,
}
impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            source_bytes: 64 * 1024,
            steps: 10_000,
            workspace_bytes: vibe::SENTINEL_BYTES,
            result_bytes: 64 * 1024,
            extension_output_bytes: 4096,
            receipts: 64,
        }
    }
}

#[derive(Debug)]
pub enum HostError {
    InvalidLimits,
    MissingQualiaFoundation,
    Graph(qualia_core_db::vibe_host::native_authority::graph::GraphError),
    SourceTooLarge,
    ResultTooLarge,
    QueueFull,
    Closed,
    Cancelled,
    WrongRuntime,
    MissingCapability(String),
    UnsupportedCapabilityConstraints(String),
    Trap(VibeTrap),
    Diagnostic(vibe::Diagnostic),
    /// Execution reached the engine. Consult the real host's journal even when
    /// this list is empty: legacy Host effects are not extension receipts.
    Reconcile {
        cause: Box<HostError>,
        committed_receipts: Vec<ExecutionReceiptHeader>,
    },
}
impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for HostError {}

/// A checked program is tied to the runtime and UI epoch that admitted it.
/// Source requests never mint grants. The host supplies granted capability IDs.
pub struct PreparedProgram {
    program: Program,
    epoch: u64,
    owner: Arc<()>,
}

/// Own on a script thread, supplying the actual QualiaDB services at startup.
/// No runtime, worker thread, GPU resource, or mock service is created implicitly.
pub struct QualiaRuntime<H, G> {
    host: H,
    gate: G,
    grants: Vec<String>,
    limits: ExecutionLimits,
    ui: UiSender,
    owner: Arc<()>,
}

impl<H: Host, G: SentinelPolicyGate> QualiaRuntime<H, G> {
    pub fn new(
        host: H,
        gate: G,
        granted_capabilities: Vec<String>,
        limits: ExecutionLimits,
        ui: UiSender,
    ) -> Result<Self, HostError> {
        if limits.source_bytes == 0
            || limits.steps == 0
            || limits.workspace_bytes == 0
            || limits.result_bytes == 0
            || limits.extension_output_bytes == 0
            || limits.receipts == 0
        {
            return Err(HostError::InvalidLimits);
        }
        if !gate.is_controller_alive() {
            return Err(HostError::Trap(VibeTrap::SentinelUnavailable));
        }
        ui.begin()?;
        Ok(Self {
            host,
            gate,
            grants: granted_capabilities,
            limits,
            ui,
            owner: Arc::new(()),
        })
    }

    /// Reuse the canonical source-session ingress limits before loading a program.
    /// This cold path parses twice because the canonical checked submission does
    /// not expose its program or accept extension governance for execution.
    pub fn prepare(&self, source: &str) -> Result<PreparedProgram, HostError> {
        let epoch = self.ui.begin()?;
        if source.len() > self.limits.source_bytes {
            return Err(HostError::SourceTooLarge);
        }
        let ingress = vibe::session::SourceSession::new(
            self.grants.clone(),
            vibe::session::SessionLimits {
                submission_bytes: self.limits.source_bytes,
                retained_source_bytes: self.limits.source_bytes,
                ..Default::default()
            },
        );
        // Its public prepare calls canonical source_limits::check before parsing,
        // including the nesting/path limits. No host operation executes here.
        ingress.prepare(source).map_err(HostError::Diagnostic)?;
        let program = vibe::load_program(source).map_err(HostError::Diagnostic)?;
        for requested in &program.requires {
            if !requested.args.is_empty() {
                return Err(HostError::UnsupportedCapabilityConstraints(
                    requested.id.clone(),
                ));
            }
            if !self.grants.contains(&requested.id) {
                return Err(HostError::MissingCapability(requested.id.clone()));
            }
        }
        Ok(PreparedProgram {
            program,
            epoch,
            owner: self.owner.clone(),
        })
    }

    /// Run once with fresh local bindings and publish only to the current UI epoch.
    /// `context` is host-authenticated; never construct it from untrusted page fields.
    /// Gate admission applies to canonical extensions; the Host must authorize its
    /// own graph/pulse/capability methods. Earlier external effects may survive error.
    pub fn execute(
        &mut self,
        prepared: PreparedProgram,
        context: CallExecutionContext,
    ) -> Result<(), HostError> {
        if !Arc::ptr_eq(&prepared.owner, &self.owner) {
            return Err(HostError::WrongRuntime);
        }
        if !self.ui.current(prepared.epoch) {
            return Err(HostError::Cancelled);
        }
        self.ui.begin()?;
        if context.mode != ExecutionMode::GovernedApplication {
            return Err(HostError::Trap(VibeTrap::EffectNotPermittedInMode));
        }
        if !self.gate.is_controller_alive() {
            return Err(HostError::Trap(VibeTrap::SentinelUnavailable));
        }
        let mut gate = ScopedGate {
            inner: &mut self.gate,
            ui: &self.ui,
            epoch: prepared.epoch,
            max_receipts: self.limits.receipts,
            ticks_left: self.limits.steps,
            max_output_bytes: self.limits.extension_output_bytes,
            receipts: Vec::new(),
        };
        let budget = Budget {
            steps_left: self.limits.steps,
            workspace_left: self.limits.workspace_bytes,
        };
        let result = {
            let mut engine = Engine::with_program(&mut self.host, budget, &prepared.program);
            engine.set_extension_governance(context, Some(&mut gate));
            engine.eval_program(&prepared.program, &mut Env::default())
        };
        let receipts = std::mem::take(&mut gate.receipts);
        drop(gate);
        let value = result.map_err(|diagnostic| HostError::Reconcile {
            cause: Box::new(HostError::Diagnostic(diagnostic)),
            committed_receipts: receipts.clone(),
        })?;
        if !self.ui.current(prepared.epoch) {
            return Err(HostError::Reconcile {
                cause: Box::new(HostError::Cancelled),
                committed_receipts: receipts,
            });
        }
        if !fits_debug(&value, self.limits.result_bytes) {
            return Err(HostError::Reconcile {
                cause: Box::new(HostError::ResultTooLarge),
                committed_receipts: receipts,
            });
        }
        // Keep receipt evidence if UI teardown races the final publication.
        self.ui
            .publish(
                prepared.epoch,
                UiDelivery {
                    value,
                    receipts: receipts.clone(),
                },
            )
            .map_err(|cause| HostError::Reconcile {
                cause: Box::new(cause),
                committed_receipts: receipts,
            })
    }
    /// Return services for authoritative reconciliation after shutting down runtime ownership.
    pub fn into_services(self) -> (H, G) {
        (self.host, self.gate)
    }
}

pub(crate) fn fits_debug(value: &impl fmt::Debug, limit: usize) -> bool {
    struct Counter(usize);
    impl Write for Counter {
        fn write_str(&mut self, value: &str) -> fmt::Result {
            self.0 = self.0.checked_sub(value.len()).ok_or(fmt::Error)?;
            Ok(())
        }
    }
    write!(Counter(limit), "{value:?}").is_ok()
}
