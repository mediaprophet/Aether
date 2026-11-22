// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Test-only peer for real private-pipe and parent deadline tests. Not Chora.
use protocol::*;
use std::{thread, time::Duration};
#[path = "../../../chora-worker/src/private_input.rs"]
mod private_input;
#[path = "../../../chora-worker/src/private_pipe.rs"]
mod private_pipe;
fn main() {
    let mut output = private_pipe::private_output().unwrap();
    let input = private_input::input_thread();
    println!("transport fixture diagnostic: stdout is not the protocol pipe");
    let mut sequence = Sequence::default();
    let mut dimensions = (1, 1);
    while let Ok(Ok(WireMessage::Control(envelope))) = input.recv() {
        let mut stamp = envelope.stamp;
        let message = match envelope.message {
            Control::Initialize { .. } => Control::Ready {
                profile: ExecutionProfile::TrustedEmbedding,
            },
            Control::Resize { width, height } => {
                dimensions = (width, height);
                Control::Invalidate {
                    reason: ReplyError::Cancelled,
                }
            }
            Control::Navigate { url } => {
                if url.ends_with("/diagnostic-burst") {
                    use std::io::Write;
                    // Exceeds a pipe's capacity: delivery must keep draining
                    // diagnostics while waiting for the next control response.
                    std::io::stderr().write_all(&[b'x'; 256 * 1024]).unwrap();
                }
                if url.ends_with("/wrong-session") {
                    stamp.session = [42; 16];
                }
                stamp.document += 1;
                Control::Invalidate {
                    reason: ReplyError::Cancelled,
                }
            }
            Control::Capture { capture_id } => {
                let (width, height) = dimensions;
                let rgba = vec![127; checked_frame_layout(width, height).unwrap().rgba_bytes];
                write_message(
                    &mut output,
                    &WireMessage::Frame(Frame {
                        stamp,
                        sequence: sequence.next().unwrap(),
                        capture_id,
                        width,
                        height,
                        rgba,
                    }),
                )
                .unwrap();
                continue;
            }
            // Deliberately uncooperative engine destruction: parent must kill/reap.
            Control::Shutdown => {
                thread::sleep(Duration::from_secs(60));
                break;
            }
            _ => continue,
        };
        write_message(
            &mut output,
            &WireMessage::Control(ControlEnvelope {
                stamp,
                sequence: sequence.next().unwrap(),
                message,
            }),
        )
        .unwrap();
    }
}
