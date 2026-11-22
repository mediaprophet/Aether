// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
use serde::{Deserialize, Serialize};
use std::fmt;

pub type Nonce = [u8; 16];
pub const MAX_CONTROL_BYTES: usize = 128 * 1024;
pub const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_CHANNEL_BYTES: usize = 128;
pub const MAX_PAYLOAD_BYTES: usize = 16 * 1024;
pub const MAX_REPLY_BYTES: usize = 64 * 1024;
pub const MAX_PENDING: usize = 64;
pub const MAX_DEADLINE_SECS: u64 = 30;
pub const MAX_ID_BYTES: usize = 128;
pub const MAX_ORIGIN_BYTES: usize = 2048;
pub const MAX_URL_BYTES: usize = 8192;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ExecutionProfile {
    TrustedEmbedding,
    RequireSandbox,
}

/// Session is parent-generated private-channel routing, never a project grant.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RouteStamp {
    pub session: Nonce,
    pub view: u64,
    pub generation: u64,
    pub document: u64,
    pub viewport: u64,
}

/// Representation of engine-supplied caller data. Deserialization adds no trust.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CallerContext {
    pub engine_instance: Nonce,
    pub webview: String,
    pub pipeline: String,
    pub navigation_epoch: Nonce,
    pub origin: String,
    pub private_browsing: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ReplyError {
    Denied,
    Cancelled,
    TimedOut,
    Unavailable,
    TooLarge,
    InvalidContext,
    Busy,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WorkerFailure {
    UnsupportedSandbox,
    Initialization,
    Rendering,
    Protocol,
    ShutdownDeadline,
}

pub const MAX_KEY_BYTES: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerAction {
    Move,
    Down,
    Up,
    Leave,
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerButton {
    Main,
    Auxiliary,
    Secondary,
    Fourth,
    Fifth,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WirePointerEvent {
    pub action: PointerAction,
    pub x_px: f32,
    pub y_px: f32,
    pub button: Option<PointerButton>,
    pub buttons: u8,
    pub modifiers: InputModifiers,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireWheelEvent {
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_is_lines: bool,
    pub x_px: f32,
    pub y_px: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireKeyState {
    Down,
    Up,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireKeyEvent {
    pub state: WireKeyState,
    pub key: String,
    pub code: String,
    pub location: u8,
    pub modifiers: InputModifiers,
    pub repeat: bool,
    pub is_composing: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum WireImeEvent {
    Preedit {
        text: String,
        cursor_start: Option<u32>,
        cursor_end: Option<u32>,
    },
    Commit {
        text: String,
    },
    Dismiss,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data", deny_unknown_fields)]
pub enum Control {
    Initialize {
        profile: ExecutionProfile,
        private_browsing: bool,
    },
    Ready {
        profile: ExecutionProfile,
    },
    Navigate {
        url: String,
    },
    Resize {
        width: u32,
        height: u32,
    },
    Capture {
        capture_id: u64,
    },
    PageRequest {
        id: Nonce,
        caller: CallerContext,
        channel: String,
        payload: String,
    },
    PageReply {
        id: Nonce,
        result: Result<String, ReplyError>,
    },
    Cancel {
        id: Nonce,
        reason: ReplyError,
    },
    Invalidate {
        reason: ReplyError,
    },
    InputPointer(WirePointerEvent),
    InputWheel(WireWheelEvent),
    InputKey(WireKeyEvent),
    InputIme(WireImeEvent),
    SetFocus {
        focused: bool,
    },
    Shutdown,
    Stopped,
    Failed {
        reason: WorkerFailure,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ControlEnvelope {
    pub stamp: RouteStamp,
    pub sequence: u64,
    pub message: Control,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Frame {
    pub stamp: RouteStamp,
    pub sequence: u64,
    pub capture_id: u64,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum WireMessage {
    Control(ControlEnvelope),
    Frame(Frame),
}
impl WireMessage {
    pub fn stamp(&self) -> RouteStamp {
        match self {
            Self::Control(c) => c.stamp,
            Self::Frame(f) => f.stamp,
        }
    }
    pub fn sequence(&self) -> u64 {
        match self {
            Self::Control(c) => c.sequence,
            Self::Frame(f) => f.sequence,
        }
    }
    pub fn validate(&self) -> Result<(), ProtocolError> {
        if self.sequence() == 0 {
            return Err(ProtocolError::Replay);
        }
        match self {
            Self::Control(c) => c.validate(),
            Self::Frame(f) => {
                let layout = checked_frame_layout(f.width, f.height)?;
                if layout.rgba_bytes != f.rgba.len() {
                    return Err(ProtocolError::InvalidDimensions);
                }
                Ok(())
            }
        }
    }
}
impl ControlEnvelope {
    pub fn validate(&self) -> Result<(), ProtocolError> {
        match &self.message {
            Control::Navigate { url } if url.is_empty() || url.len() > MAX_URL_BYTES => {
                return Err(ProtocolError::TooLarge);
            }
            Control::Resize { width, height } => {
                checked_frame_layout(*width, *height)?;
            }
            Control::PageRequest {
                caller,
                channel,
                payload,
                ..
            } => {
                if channel.len() > MAX_CHANNEL_BYTES
                    || payload.len() > MAX_PAYLOAD_BYTES
                    || caller.webview.len() > MAX_ID_BYTES
                    || caller.pipeline.len() > MAX_ID_BYTES
                    || caller.origin.len() > MAX_ORIGIN_BYTES
                {
                    return Err(ProtocolError::TooLarge);
                }
                if caller.webview.is_empty()
                    || caller.pipeline.is_empty()
                    || caller.origin.is_empty()
                    || channel.is_empty()
                    || !channel
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
                {
                    return Err(ProtocolError::InvalidMessage);
                }
            }
            Control::PageReply {
                result: Ok(body), ..
            } if body.len() > MAX_REPLY_BYTES => return Err(ProtocolError::TooLarge),
            Control::InputPointer(event) => {
                if !event.x_px.is_finite()
                    || !event.y_px.is_finite()
                    || event.buttons & !0b1_1111 != 0
                    || (matches!(event.action, PointerAction::Down | PointerAction::Up)
                        && event.button.is_none())
                {
                    return Err(ProtocolError::InvalidMessage);
                }
            }
            Control::InputWheel(event) => {
                if !event.delta_x.is_finite()
                    || !event.delta_y.is_finite()
                    || !event.x_px.is_finite()
                    || !event.y_px.is_finite()
                {
                    return Err(ProtocolError::InvalidMessage);
                }
            }
            Control::InputKey(event) => {
                if event.key.len() > MAX_KEY_BYTES || event.code.len() > MAX_KEY_BYTES {
                    return Err(ProtocolError::TooLarge);
                }
                if event.location > 3 || event.key.is_empty() || event.code.is_empty() {
                    return Err(ProtocolError::InvalidMessage);
                }
            }
            Control::InputIme(event) => match event {
                WireImeEvent::Dismiss => {}
                WireImeEvent::Commit { text } => {
                    if text.len() > MAX_PAYLOAD_BYTES {
                        return Err(ProtocolError::TooLarge);
                    }
                }
                WireImeEvent::Preedit {
                    text,
                    cursor_start,
                    cursor_end,
                } => {
                    if text.len() > MAX_PAYLOAD_BYTES {
                        return Err(ProtocolError::TooLarge);
                    }
                    match (cursor_start, cursor_end) {
                        (None, None) => {}
                        (Some(start), Some(end))
                            if start <= end
                                && text.is_char_boundary(*start as usize)
                                && text.is_char_boundary(*end as usize) => {}
                        _ => return Err(ProtocolError::InvalidMessage),
                    }
                }
            },
            _ => {}
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameLayout {
    pub rgba_bytes: usize,
    pub readback_reservation_bytes: usize,
}
/// Admit before Chora readback: reserve four pixel buffers conservatively for
/// readback/flip/transfer scratch. This excludes engine heap and GPU surfaces.
pub fn checked_frame_layout(width: u32, height: u32) -> Result<FrameLayout, ProtocolError> {
    if width == 0 || height == 0 {
        return Err(ProtocolError::InvalidDimensions);
    }
    let rgba_bytes = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .filter(|n| *n <= MAX_FRAME_BYTES)
        .ok_or(ProtocolError::TooLarge)?;
    let readback_reservation_bytes = rgba_bytes.checked_mul(4).ok_or(ProtocolError::TooLarge)?;
    Ok(FrameLayout {
        rgba_bytes,
        readback_reservation_bytes,
    })
}

#[derive(Debug)]
pub enum ProtocolError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidHeader,
    InvalidMessage,
    TooLarge,
    Allocation,
    InvalidDimensions,
    WrongSession,
    WrongDirection,
    Replay,
    Closed,
    Busy,
    StaleFrame,
    SequenceExhausted,
}
impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Io(_) => "private channel I/O failed",
            Self::Json(_) => "invalid control JSON",
            Self::InvalidHeader => "invalid protocol header",
            Self::InvalidMessage => "invalid control message",
            Self::TooLarge => "transport limit exceeded",
            Self::Allocation => "transport allocation failed",
            Self::InvalidDimensions => "invalid frame dimensions",
            Self::WrongSession => "wrong worker session",
            Self::WrongDirection => "unexpected message direction",
            Self::Replay => "out-of-order message",
            Self::Closed => "worker route closed",
            Self::Busy => "bounded slot is occupied",
            Self::StaleFrame => "stale frame",
            Self::SequenceExhausted => "sequence exhausted",
        })
    }
}
impl std::error::Error for ProtocolError {}
impl From<std::io::Error> for ProtocolError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<serde_json::Error> for ProtocolError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
