// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Concrete required QualiaDB foundation for the supported native effect profile.
use crate::{ExecutionLimits, HostError, QualiaRuntime, UiSender};
use qualia_core_db::vibe_host::{
    PoetSnapshot, scoped_effects::ScopedEffects, vibe_sentinel::WebizenSentinelGate,
};

/// Real QualiaDB services, borrowing native owner-held state and policy buffers.
pub type NativeScopedRuntime<'host, 'policy> =
    QualiaRuntime<ScopedEffects<'host>, WebizenSentinelGate<'policy>>;

/// Compose the canonical scoped host and Sentinel within a native admission scope.
///
/// The native owner must authenticate/admit the subject, retain this borrow inside
/// that transaction, and supply adopted policy/grants. Use `PoetSnapshot::from_daemon`
/// after actual daemon initialization. `PoetSnapshot::live` is unsuitable because
/// it can silently substitute demo data. A detached snapshot is rejected here.
///
/// The current canonical scoped profile supports pulse.publish, hid.poll,
/// crypto.generate_key and crypto.sign; it denies graph reads/writes and generic
/// capability invocation. This function does not broaden its grants.
///
/// `WebizenSentinelGate` receipts go to caller-owned RAM NQuins, not a durable
/// journal. NativeAuthority's signed-admission journal is a separate contract.
/// This concrete composition is not yet a durable application release profile.
pub fn scoped_native_runtime<'host, 'policy>(
    snapshot: &'host mut PoetSnapshot,
    grants: &'host [&'host str],
    gate: WebizenSentinelGate<'policy>,
    limits: ExecutionLimits,
    ui: UiSender,
) -> Result<NativeScopedRuntime<'host, 'policy>, HostError> {
    if !snapshot.attached {
        return Err(HostError::MissingQualiaFoundation);
    }
    let host = ScopedEffects::new(snapshot, grants).map_err(HostError::Diagnostic)?;
    QualiaRuntime::new(
        host,
        gate,
        grants.iter().map(|id| (*id).to_owned()).collect(),
        limits,
        ui,
    )
}
