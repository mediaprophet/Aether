// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
use crate::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PeerRole {
    Parent,
    Worker,
}

#[derive(Debug, Default)]
pub struct Sequence(u64);
impl Sequence {
    /// Exhaustion is terminal; sequence zero is never issued.
    pub fn next(&mut self) -> Result<u64, ProtocolError> {
        self.0 = self
            .0
            .checked_add(1)
            .ok_or(ProtocolError::SequenceExhausted)?;
        Ok(self.0)
    }
}

/// A verified pipe message is session-bound, direction-checked and ordered. It
/// is still untrusted data: it is neither canonical authority nor a route grant.
#[derive(Debug)]
pub struct VerifiedMessage(WireMessage);
impl VerifiedMessage {
    pub fn message(&self) -> &WireMessage {
        &self.0
    }
    pub fn into_inner(self) -> WireMessage {
        self.0
    }
}

pub struct PeerGuard {
    role: PeerRole,
    session: Nonce,
    received: u64,
    closed: bool,
}
impl PeerGuard {
    pub fn new(role: PeerRole, session: Nonce) -> Self {
        Self {
            role,
            session,
            received: 0,
            closed: false,
        }
    }
    pub fn verify(&mut self, message: WireMessage) -> Result<VerifiedMessage, ProtocolError> {
        if self.closed {
            return Err(ProtocolError::Closed);
        }
        let result = self.check(&message);
        if let Err(error) = result {
            self.closed = true;
            return Err(error);
        }
        self.received = message.sequence();
        Ok(VerifiedMessage(message))
    }
    fn check(&self, message: &WireMessage) -> Result<(), ProtocolError> {
        message.validate()?;
        if message.stamp().session != self.session {
            return Err(ProtocolError::WrongSession);
        }
        if message.sequence() <= self.received {
            return Err(ProtocolError::Replay);
        }
        let from_parent = match message {
            WireMessage::Frame(_) => false,
            WireMessage::Control(c) => matches!(
                c.message,
                Control::Initialize { .. }
                    | Control::Navigate { .. }
                    | Control::Resize { .. }
                    | Control::Capture { .. }
                    | Control::PageReply { .. }
                    | Control::InputPointer(_)
                    | Control::InputWheel(_)
                    | Control::InputKey(_)
                    | Control::InputIme(_)
                    | Control::SetFocus { .. }
                    | Control::Shutdown
            ),
        };
        // Both peers may cancel a retained request; neither cancellation changes
        // the route's generation/document identity.
        let cancel = matches!(
            message,
            WireMessage::Control(ControlEnvelope {
                message: Control::Cancel { .. },
                ..
            })
        );
        if !cancel && from_parent != (self.role == PeerRole::Worker) {
            return Err(ProtocolError::WrongDirection);
        }
        Ok(())
    }
    pub fn close(&mut self) {
        self.closed = true;
    }
}

/// Exactly one outstanding capture and one latest frame. Navigation, actual URL
/// invalidation and resize must supply the complete replacement stamp.
pub struct FrameMailbox {
    stamp: RouteStamp,
    capture: Option<u64>,
    latest: Option<Frame>,
    closed: bool,
}
impl FrameMailbox {
    pub fn new(stamp: RouteStamp) -> Self {
        Self {
            stamp,
            capture: None,
            latest: None,
            closed: false,
        }
    }
    pub fn begin_capture(&mut self, id: u64) -> Result<(), ProtocolError> {
        if self.closed {
            return Err(ProtocolError::Closed);
        }
        if self.capture.is_some() {
            return Err(ProtocolError::Busy);
        }
        self.capture = Some(id);
        Ok(())
    }
    pub fn publish(&mut self, frame: Frame) -> Result<(), ProtocolError> {
        if self.closed {
            return Err(ProtocolError::Closed);
        }
        if frame.stamp != self.stamp || self.capture != Some(frame.capture_id) {
            return Err(ProtocolError::StaleFrame);
        }
        let layout = checked_frame_layout(frame.width, frame.height)?;
        if frame.sequence == 0 {
            return Err(ProtocolError::Replay);
        }
        if layout.rgba_bytes != frame.rgba.len() {
            return Err(ProtocolError::InvalidDimensions);
        }
        self.capture = None;
        self.latest = Some(frame);
        Ok(())
    }
    pub fn take(&mut self) -> Option<Frame> {
        self.latest.take()
    }
    pub fn invalidate(&mut self, stamp: RouteStamp) {
        self.stamp = stamp;
        self.capture = None;
        self.latest = None;
    }
    pub fn cancel_capture(&mut self) {
        self.capture = None;
    }
    pub fn close(&mut self) {
        self.closed = true;
        self.capture = None;
        self.latest = None;
    }
}
