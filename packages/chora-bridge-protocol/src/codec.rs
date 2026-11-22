// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
use crate::*;
use std::io::{Read, Write};

const MAGIC: &[u8; 4] = b"CHOR";
const VERSION: u16 = 1;
const FRAME_HEADER: usize = 72;

/// A versioned header bounds allocation before payload decoding. Framing errors
/// are terminal for the pipe; callers must close it rather than resynchronize.
pub fn read_message(reader: &mut impl Read) -> Result<WireMessage, ProtocolError> {
    let mut header = [0; 12];
    reader.read_exact(&mut header)?;
    if &header[..4] != MAGIC
        || u16::from_le_bytes([header[4], header[5]]) != VERSION
        || header[7] != 0
        || !matches!(header[6], 1 | 2)
    {
        return Err(ProtocolError::InvalidHeader);
    }
    let len = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;
    let limit = if header[6] == 1 {
        MAX_CONTROL_BYTES
    } else {
        MAX_FRAME_BYTES + FRAME_HEADER
    };
    if len == 0 || len > limit {
        return Err(ProtocolError::TooLarge);
    }
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(len)
        .map_err(|_| ProtocolError::Allocation)?;
    payload.resize(len, 0);
    reader.read_exact(&mut payload)?;
    let message = if header[6] == 1 {
        WireMessage::Control(serde_json::from_slice(&payload)?)
    } else {
        if len < FRAME_HEADER {
            return Err(ProtocolError::InvalidHeader);
        }
        let u64_at = |n| u64::from_le_bytes(payload[n..n + 8].try_into().unwrap());
        let stamp = RouteStamp {
            session: payload[..16].try_into().unwrap(),
            view: u64_at(16),
            generation: u64_at(24),
            document: u64_at(32),
            viewport: u64_at(40),
        };
        let width = u32::from_le_bytes(payload[64..68].try_into().unwrap());
        let height = u32::from_le_bytes(payload[68..72].try_into().unwrap());
        let layout = checked_frame_layout(width, height)?;
        if len - FRAME_HEADER != layout.rgba_bytes {
            return Err(ProtocolError::InvalidDimensions);
        }
        // Reuse the bounded allocation; no second full pixel buffer is needed.
        let sequence = u64_at(48);
        let capture_id = u64_at(56);
        payload.copy_within(FRAME_HEADER.., 0);
        payload.truncate(layout.rgba_bytes);
        WireMessage::Frame(Frame {
            stamp,
            sequence,
            capture_id,
            width,
            height,
            rgba: payload,
        })
    };
    message.validate()?;
    Ok(message)
}

pub fn write_message(writer: &mut impl Write, message: &WireMessage) -> Result<(), ProtocolError> {
    message.validate()?;
    let (kind, len, control) = match message {
        WireMessage::Control(envelope) => {
            let mut encoded = ControlBytes {
                bytes: Vec::new(),
                overflow: false,
            };
            encoded
                .bytes
                .try_reserve_exact(MAX_CONTROL_BYTES)
                .map_err(|_| ProtocolError::Allocation)?;
            if let Err(error) = serde_json::to_writer(&mut encoded, envelope) {
                return Err(if encoded.overflow {
                    ProtocolError::TooLarge
                } else {
                    ProtocolError::Json(error)
                });
            }
            (1, encoded.bytes.len(), Some(encoded.bytes))
        }
        WireMessage::Frame(frame) => (2, FRAME_HEADER + frame.rgba.len(), None),
    };
    let mut header = [0; 12];
    header[..4].copy_from_slice(MAGIC);
    header[4..6].copy_from_slice(&VERSION.to_le_bytes());
    header[6] = kind;
    header[8..12].copy_from_slice(&(len as u32).to_le_bytes());
    writer.write_all(&header)?;
    match message {
        WireMessage::Control(_) => writer.write_all(control.as_ref().unwrap())?,
        WireMessage::Frame(frame) => {
            writer.write_all(&frame.stamp.session)?;
            for value in [
                frame.stamp.view,
                frame.stamp.generation,
                frame.stamp.document,
                frame.stamp.viewport,
                frame.sequence,
                frame.capture_id,
            ] {
                writer.write_all(&value.to_le_bytes())?;
            }
            writer.write_all(&frame.width.to_le_bytes())?;
            writer.write_all(&frame.height.to_le_bytes())?;
            writer.write_all(&frame.rgba)?;
        }
    }
    writer.flush()?;
    Ok(())
}

// Escaping can expand JSON strings, so bound serialized bytes while writing,
// before touching the pipe or allocating beyond the control-frame limit.
struct ControlBytes {
    bytes: Vec<u8>,
    overflow: bool,
}
impl Write for ControlBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > MAX_CONTROL_BYTES.saturating_sub(self.bytes.len()) {
            self.overflow = true;
            return Err(std::io::ErrorKind::WriteZero.into());
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
