// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Transport representations for a parent-owned private pipe, not credentials,
//! a scripting ABI, browser policy, or OS containment. The worker derives caller
//! data only from Chora's real request; the parent separately authorizes services.
//! Frames are copied readback RGBA8. Alpha/orientation require runtime qualification.

mod codec;
mod state;
mod types;

pub use codec::{read_message, write_message};
pub use state::{FrameMailbox, PeerGuard, PeerRole, Sequence, VerifiedMessage};
pub use types::*;

#[cfg(test)]
mod tests;
