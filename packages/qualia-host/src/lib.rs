// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Host-side VibeScript execution and native UI delivery using QualiaDB contracts.
//!
//! The application MUST supply its real QualiaDB [`vibe::Host`] and authoritative
//! [`vibescript_abi::SentinelPolicyGate`]. There is no default host or policy gate.
//! This crate cannot authenticate an arbitrary Rust implementation of either trait.
//! Native host calls remain governed by that host; extension calls use the canonical
//! registry lifecycle. No JavaScript evaluator, database substitute, extension ABI,
//! GPU device, or renderer lives here.
//!
//! Cancellation prevents subsequent extension admission and UI publication. It is
//! cooperative: an executing provider or host call is not preempted, and effects
//! already committed are not rolled back. See the crate README for release gates.

mod authoring;
#[cfg(test)]
mod authoring_tests;
mod chora;
mod foundation;
mod governance;
mod graph;
pub mod holographic_viewport;
pub mod hooks;
mod mailbox;
#[cfg(test)]
mod mailbox_tests;
mod native_graph_ui;
pub mod native_ui;
mod runtime;
#[cfg(test)]
mod tests;

pub use aether_chora_bridge_protocol as chora_protocol;
pub use authoring::{AuthoringBridgeError, NativeAuthoringBridge, NativeAuthoringView};
pub use chora::ChoraGraphRoutes;
pub use foundation::{NativeScopedRuntime, scoped_native_runtime};
pub use graph::{
    GraphSnapshot, NativeGraphBridge, NativeGraphRuntime, NativeGraphView, native_graph_runtime,
};
pub use mailbox::{UiDelivery, UiReceiver, UiSender, ui_channel};
pub use qualia_cooperative_core;
pub use qualia_core_db;
pub use runtime::{ExecutionLimits, HostError, PreparedProgram, QualiaRuntime};
pub use vibe;
pub use vibescript_abi;

/// Native host contexts, actions, graph observations and source review.
pub mod prelude {
    pub use crate::holographic_viewport::{
        Holographic10dViewportSpec, Holographic10dViewportState,
    };
    pub use crate::hooks::*;
}
