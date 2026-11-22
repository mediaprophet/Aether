// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
use crate::*;

fn stamp() -> RouteStamp {
    RouteStamp {
        session: [7; 16],
        view: 1,
        generation: 2,
        document: 3,
        viewport: 4,
    }
}
fn control(sequence: u64, message: Control) -> WireMessage {
    WireMessage::Control(ControlEnvelope {
        stamp: stamp(),
        sequence,
        message,
    })
}
fn frame(id: u64) -> Frame {
    Frame {
        stamp: stamp(),
        sequence: 2,
        capture_id: id,
        width: 2,
        height: 1,
        rgba: vec![1, 2, 3, 4, 5, 6, 7, 8],
    }
}

#[test]
fn round_trip_control_and_binary_frame() {
    let messages = [
        control(
            1,
            Control::Ready {
                profile: ExecutionProfile::TrustedEmbedding,
            },
        ),
        WireMessage::Frame(frame(9)),
    ];
    let mut bytes = Vec::new();
    for message in &messages {
        write_message(&mut bytes, message).unwrap();
    }
    let mut reader = bytes.as_slice();
    for message in messages {
        assert_eq!(read_message(&mut reader).unwrap(), message);
    }
    assert!(reader.is_empty());
}

#[test]
fn rejects_header_versions_reserved_bits_and_lengths_before_payload() {
    for header in [
        b"CHOR\x02\x00\x01\x00\x01\x00\x00\x00".as_slice(),
        b"CHOR\x01\x00\x01\x01\x01\x00\x00\x00",
        b"CHOR\x01\x00\x01\x00\xff\xff\xff\xff",
    ] {
        assert!(matches!(
            read_message(&mut &header[..]),
            Err(ProtocolError::InvalidHeader | ProtocolError::TooLarge)
        ));
    }
}

#[test]
fn rejects_truncation_unknown_fields_and_mismatched_pixels() {
    let mut bytes = Vec::new();
    write_message(&mut bytes, &WireMessage::Frame(frame(3))).unwrap();
    bytes.pop();
    assert!(matches!(
        read_message(&mut bytes.as_slice()),
        Err(ProtocolError::Io(_))
    ));
    let mut bad = frame(3);
    bad.rgba.pop();
    assert!(matches!(
        write_message(&mut Vec::new(), &WireMessage::Frame(bad)),
        Err(ProtocolError::InvalidDimensions)
    ));
    let json = serde_json::to_value(match control(1, Control::Shutdown) {
        WireMessage::Control(c) => c,
        _ => unreachable!(),
    })
    .unwrap();
    let mut object = json.as_object().unwrap().clone();
    object.insert("authority".into(), true.into());
    assert!(serde_json::from_value::<ControlEnvelope>(object.into()).is_err());
}

#[test]
fn guards_direction_session_replay_and_poison() {
    let mut guard = PeerGuard::new(PeerRole::Parent, stamp().session);
    guard
        .verify(control(
            2,
            Control::Ready {
                profile: ExecutionProfile::TrustedEmbedding,
            },
        ))
        .unwrap();
    assert!(matches!(
        guard.verify(control(2, Control::Stopped)),
        Err(ProtocolError::Replay)
    ));
    assert!(matches!(
        guard.verify(control(3, Control::Stopped)),
        Err(ProtocolError::Closed)
    ));
    assert!(matches!(
        PeerGuard::new(PeerRole::Parent, stamp().session).verify(control(1, Control::Shutdown)),
        Err(ProtocolError::WrongDirection)
    ));
    assert!(matches!(
        PeerGuard::new(PeerRole::Worker, [0; 16]).verify(control(1, Control::Shutdown)),
        Err(ProtocolError::WrongSession)
    ));
    assert!(
        PeerGuard::new(PeerRole::Worker, stamp().session)
            .verify(control(
                1,
                Control::Cancel {
                    id: [0; 16],
                    reason: ReplyError::Cancelled
                }
            ))
            .is_ok()
    );
}

#[test]
fn capture_and_latest_frame_are_bounded_and_exact() {
    let mut mailbox = FrameMailbox::new(stamp());
    mailbox.begin_capture(9).unwrap();
    assert!(matches!(
        mailbox.begin_capture(10),
        Err(ProtocolError::Busy)
    ));
    assert!(matches!(
        mailbox.publish(frame(10)),
        Err(ProtocolError::StaleFrame)
    ));
    mailbox.publish(frame(9)).unwrap();
    mailbox.begin_capture(10).unwrap();
    mailbox.publish(frame(10)).unwrap();
    assert_eq!(mailbox.take().unwrap().capture_id, 10);
    assert!(mailbox.take().is_none());
    mailbox.begin_capture(11).unwrap();
    let mut next = stamp();
    next.viewport += 1;
    mailbox.invalidate(next);
    assert!(matches!(
        mailbox.publish(frame(11)),
        Err(ProtocolError::StaleFrame)
    ));
    mailbox.close();
    assert!(matches!(
        mailbox.begin_capture(12),
        Err(ProtocolError::Closed)
    ));
}

#[test]
fn dimensions_and_control_limits_are_checked() {
    assert!(checked_frame_layout(0, 1).is_err());
    assert!(checked_frame_layout(u32::MAX, u32::MAX).is_err());
    assert_eq!(
        checked_frame_layout(2048, 2048).unwrap().rgba_bytes,
        MAX_FRAME_BYTES
    );
    assert!(
        control(
            1,
            Control::Navigate {
                url: "x".repeat(MAX_URL_BYTES + 1)
            }
        )
        .validate()
        .is_err()
    );
    assert!(
        control(
            1,
            Control::PageReply {
                id: [0; 16],
                result: Ok("x".repeat(MAX_REPLY_BYTES + 1))
            }
        )
        .validate()
        .is_err()
    );
    assert!(control(0, Control::Shutdown).validate().is_err());
}

#[test]
fn json_escaping_is_bounded_before_writing_the_pipe() {
    let message = control(
        1,
        Control::PageReply {
            id: [1; 16],
            result: Ok("\0".repeat(MAX_REPLY_BYTES)),
        },
    );
    let mut pipe = Vec::new();
    assert!(matches!(
        write_message(&mut pipe, &message),
        Err(ProtocolError::TooLarge)
    ));
    assert!(pipe.is_empty());
}

#[test]
fn input_controls_require_the_parent_and_round_trip() {
    let commands = [
        Control::InputPointer(WirePointerEvent {
            action: PointerAction::Down,
            x_px: 20.0,
            y_px: 10.0,
            button: Some(PointerButton::Main),
            buttons: 1,
            modifiers: InputModifiers::default(),
        }),
        Control::InputWheel(WireWheelEvent {
            delta_x: 0.0,
            delta_y: 1.0,
            delta_is_lines: true,
            x_px: 0.0,
            y_px: 0.0,
        }),
        Control::InputKey(WireKeyEvent {
            state: WireKeyState::Down,
            key: "é".into(),
            code: "KeyE".into(),
            location: 0,
            modifiers: InputModifiers::default(),
            repeat: false,
            is_composing: false,
        }),
        Control::InputIme(WireImeEvent::Preedit {
            text: "é".into(),
            cursor_start: Some(0),
            cursor_end: Some(2),
        }),
        Control::SetFocus { focused: true },
    ];
    for command in commands {
        let packet = control(1, command.clone());
        let mut bytes = Vec::new();
        write_message(&mut bytes, &packet).unwrap();
        let incoming = read_message(&mut bytes.as_slice()).unwrap();
        assert_eq!(packet, incoming);
        let mut worker = PeerGuard::new(PeerRole::Worker, stamp().session);
        assert!(worker.verify(incoming).is_ok());
        let mut parent = PeerGuard::new(PeerRole::Parent, stamp().session);
        assert!(matches!(
            parent.verify(control(1, command)),
            Err(ProtocolError::WrongDirection)
        ));
    }
}

#[test]
fn input_validation_rejects_nonfinite_and_out_of_range_fields() {
    let commands = [
        Control::InputPointer(WirePointerEvent {
            action: PointerAction::Move,
            x_px: f32::NAN,
            y_px: 0.0,
            button: None,
            buttons: 0,
            modifiers: InputModifiers::default(),
        }),
        Control::InputWheel(WireWheelEvent {
            delta_x: 0.0,
            delta_y: f64::INFINITY,
            delta_is_lines: true,
            x_px: 0.0,
            y_px: 0.0,
        }),
        Control::InputKey(WireKeyEvent {
            state: WireKeyState::Up,
            key: "x".into(),
            code: "KeyX".into(),
            location: 4,
            modifiers: InputModifiers::default(),
            repeat: false,
            is_composing: false,
        }),
        Control::InputIme(WireImeEvent::Preedit {
            text: "é".into(),
            cursor_start: Some(1),
            cursor_end: Some(2),
        }),
    ];
    for command in commands {
        assert!(matches!(
            control(1, command).validate(),
            Err(ProtocolError::InvalidMessage)
        ));
    }
    let oversize = WireImeEvent::Commit {
        text: "x".repeat(MAX_PAYLOAD_BYTES + 1),
    };
    assert!(matches!(
        control(1, Control::InputIme(oversize)).validate(),
        Err(ProtocolError::TooLarge)
    ));
}
