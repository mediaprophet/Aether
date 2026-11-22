// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
use crate::mailbox::UiSender;
use vibescript_abi::{
    AdmissionRequest, AdmissionTicket, ExecutionMode, ExecutionReceiptHeader, NQuin48,
    SentinelPolicyGate, VibeOpCode, VibeTrap,
};

/// Enforces adapter lifecycle bounds around the application's real authority.
/// This is deliberately private: it is not an alternative policy implementation.
pub(crate) struct ScopedGate<'a, G> {
    pub inner: &'a mut G,
    pub ui: &'a UiSender,
    pub epoch: u64,
    pub max_receipts: usize,
    pub ticks_left: u64,
    pub max_output_bytes: usize,
    pub receipts: Vec<ExecutionReceiptHeader>,
}

impl<G: SentinelPolicyGate> ScopedGate<'_, G> {
    fn checkpoint(&self) -> Result<(), VibeTrap> {
        if !self.ui.current(self.epoch) {
            return Err(VibeTrap::StagingAbortRequired);
        }
        if !self.inner.is_controller_alive() {
            return Err(VibeTrap::SentinelUnavailable);
        }
        Ok(())
    }
}

impl<G: SentinelPolicyGate> SentinelPolicyGate for ScopedGate<'_, G> {
    fn is_controller_alive(&self) -> bool {
        self.ui.current(self.epoch) && self.inner.is_controller_alive()
    }
    fn current_policy_generation(&self) -> u64 {
        self.inner.current_policy_generation()
    }
    fn admit_call(&mut self, request: &AdmissionRequest) -> Result<AdmissionTicket, VibeTrap> {
        self.checkpoint()?;
        if request.mode != ExecutionMode::GovernedApplication {
            return Err(VibeTrap::EffectNotPermittedInMode);
        }
        if self.receipts.len() == self.max_receipts
            || request.declared_envelope.max_ticks > self.ticks_left
            || request.declared_envelope.max_output_bytes as usize > self.max_output_bytes
        {
            return Err(VibeTrap::ResourceEnvelopeExceeded);
        }
        let ticket = self.inner.admit_call(request)?;
        self.checkpoint()?;
        // These checks precede provider.call in the canonical dispatcher.
        if ticket.simulated {
            return Err(VibeTrap::PolicyDenied);
        }
        if ticket.actor_did_hash != request.actor_did_hash
            || ticket.context_world_hash != request.context_world_hash
        {
            return Err(VibeTrap::DelegationScopeMismatch);
        }
        if ticket.expiry_lamport < request.lamport_timestamp {
            return Err(VibeTrap::AuthorityExpired);
        }
        if ticket.policy_generation != self.inner.current_policy_generation() {
            return Err(VibeTrap::RevokedPolicyGeneration);
        }
        if !request
            .requested_effects
            .is_subset_of(ticket.granted_effects)
        {
            return Err(VibeTrap::PolicyDenied);
        }
        if !request
            .declared_envelope
            .fits_within(&ticket.approved_envelope)
        {
            return Err(VibeTrap::ResourceEnvelopeExceeded);
        }
        Ok(ticket)
    }
    fn validate_post_execution(
        &mut self,
        ticket: &AdmissionTicket,
        opcode: VibeOpCode,
        ticks: u64,
        bytes: usize,
        digest: u64,
    ) -> Result<(), VibeTrap> {
        self.checkpoint()?;
        if ticket.policy_generation != self.inner.current_policy_generation() {
            return Err(VibeTrap::MidExecutionPolicyRevoked);
        }
        if ticks > self.ticks_left || bytes > self.max_output_bytes {
            return Err(VibeTrap::ResourceEnvelopeExceeded);
        }
        self.inner
            .validate_post_execution(ticket, opcode, ticks, bytes, digest)?;
        self.ticks_left -= ticks;
        Ok(())
    }
    fn commit_receipt(
        &mut self,
        receipt: &ExecutionReceiptHeader,
    ) -> Result<[NQuin48; 2], VibeTrap> {
        self.checkpoint()?;
        if self.receipts.len() == self.max_receipts {
            return Err(VibeTrap::ResourceEnvelopeExceeded);
        }
        if receipt.simulated || receipt.mode != ExecutionMode::GovernedApplication {
            return Err(VibeTrap::PolicyDenied);
        }
        if receipt.policy_generation != self.inner.current_policy_generation() {
            return Err(VibeTrap::MidExecutionPolicyRevoked);
        }
        let audit = self.inner.commit_receipt(receipt)?;
        self.receipts.push(*receipt);
        Ok(audit)
    }
}
