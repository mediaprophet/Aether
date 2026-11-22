// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
use super::*;
fn stamp() -> RouteStamp {
    RouteStamp {
        session: [7; 16],
        view: 1,
        generation: 0,
        document: 0,
        viewport: 0,
    }
}
fn state() -> State {
    let stamp = stamp();
    State {
        stamp,
        sequence: Sequence::default(),
        mailbox: FrameMailbox::new(stamp),
        events: VecDeque::new(),
        ready: true,
        closed: false,
        exited: false,
        dimensions: (1, 1),
        capture_deadline: None,
        startup_deadline: Instant::now() + DEADLINE,
        transition_deadline: None,
        pending_move: None,
    }
}
fn frame(stamp: RouteStamp, capture_id: u64) -> WireMessage {
    WireMessage::Frame(Frame {
        stamp,
        capture_id,
        sequence: 3,
        width: 1,
        height: 1,
        rgba: vec![0, 1, 2, 255],
    })
}
#[test]
fn stopped_operations_report_closed_without_queueing() {
    for shutdown_requested in [false, true] {
        let mut state = state();
        if !shutdown_requested {
            state.close();
        }
        let (tx, rx) = mpsc::sync_channel(QUEUE);
        let client = ChoraClient(Arc::new(Owner {
            diagnostics: Arc::new(Mutex::new(WorkerDiagnostics::default())),
            state: Arc::new(Mutex::new(state)),
            output: tx,
            stop: Arc::new(AtomicBool::new(shutdown_requested)),
        }));
        assert!(matches!(
            client.navigate("about:blank".into()),
            Err(ProtocolError::Closed)
        ));
        assert!(matches!(client.resize(1, 1), Err(ProtocolError::Closed)));
        assert!(matches!(client.capture(1), Err(ProtocolError::Closed)));
        assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }
}
#[test]
fn terminal_reason_survives_shutdown_and_reaping() {
    let mut state = state();
    state.events.push_back(ControlEnvelope {
        stamp: state.stamp,
        sequence: 1,
        message: Control::Ready {
            profile: ExecutionProfile::TrustedEmbedding,
        },
    });
    state
        .receive(WireMessage::Control(ControlEnvelope {
            stamp: state.stamp,
            sequence: 2,
            message: Control::Failed {
                reason: WorkerFailure::Rendering,
            },
        }))
        .unwrap();
    state.close();
    state.exited = true;
    state.close();
    assert!(!state.ready);
    assert!(matches!(
        state.events.pop_front(),
        Some(ControlEnvelope {
            message: Control::Failed {
                reason: WorkerFailure::Rendering
            },
            ..
        })
    ));
    assert!(state.events.is_empty());
}
#[test]
fn readiness_requires_current_full_route() {
    let mut state = state();
    state.ready = false;
    let expected = state.stamp;
    for mismatched in [
        RouteStamp {
            generation: 1,
            ..expected
        },
        RouteStamp {
            document: 1,
            ..expected
        },
        RouteStamp {
            viewport: 1,
            ..expected
        },
    ] {
        state
            .receive(WireMessage::Control(ControlEnvelope {
                stamp: mismatched,
                sequence: 1,
                message: Control::Ready {
                    profile: ExecutionProfile::TrustedEmbedding,
                },
            }))
            .unwrap();
        assert!(!state.ready);
        assert!(state.events.is_empty());
    }
    state
        .receive(WireMessage::Control(ControlEnvelope {
            stamp: expected,
            sequence: 2,
            message: Control::Ready {
                profile: ExecutionProfile::TrustedEmbedding,
            },
        }))
        .unwrap();
    assert!(state.ready);
}
#[test]
fn navigation_never_delivers_old_pixels() {
    let mut state = state();
    state.mailbox.begin_capture(1).unwrap();
    let old = state.stamp;
    state.invalidate(RouteStamp {
        generation: 1,
        ..old
    });
    state.receive(frame(old, 1)).unwrap();
    assert!(state.mailbox.take().is_none());
}
#[test]
fn resize_ack_does_not_cancel_subsequent_capture() {
    let mut state = state();
    state.invalidate(RouteStamp {
        viewport: 1,
        ..state.stamp
    });
    state.mailbox.begin_capture(2).unwrap();
    state
        .receive(WireMessage::Control(ControlEnvelope {
            stamp: state.stamp,
            sequence: 2,
            message: Control::Invalidate {
                reason: ReplyError::Cancelled,
            },
        }))
        .unwrap();
    state.receive(frame(state.stamp, 2)).unwrap();
    assert!(state.mailbox.take().is_some());
}
#[test]
fn actual_document_change_cancels_old_capture_and_requests() {
    let mut state = state();
    let old = state.stamp;
    state.mailbox.begin_capture(2).unwrap();
    let new = RouteStamp { document: 1, ..old };
    state
        .receive(WireMessage::Control(ControlEnvelope {
            stamp: new,
            sequence: 2,
            message: Control::Invalidate {
                reason: ReplyError::Cancelled,
            },
        }))
        .unwrap();
    state.receive(frame(old, 2)).unwrap();
    assert_eq!(state.stamp, new);
    assert!(state.mailbox.take().is_none());
    assert_eq!(state.events.len(), 1);
}
#[test]
fn correct_stamp_does_not_admit_wrong_dimensions_or_capture() {
    let mut state = state();
    state.mailbox.begin_capture(1).unwrap();
    assert!(matches!(
        state.receive(frame(state.stamp, 2)),
        Err(ProtocolError::StaleFrame)
    ));
    state.dimensions = (2, 2);
    assert!(matches!(
        state.receive(frame(state.stamp, 1)),
        Err(ProtocolError::InvalidDimensions)
    ));
}
#[test]
fn unsupported_profile_rejected_before_spawn() {
    assert_eq!(
        ChoraClient::launch(
            Path::new("missing-worker"),
            ExecutionProfile::RequireSandbox,
            false
        )
        .err()
        .unwrap()
        .kind(),
        io::ErrorKind::Unsupported
    );
    assert_eq!(
        ChoraClient::launch(
            Path::new("missing-worker"),
            ExecutionProfile::TrustedEmbedding,
            true
        )
        .err()
        .unwrap()
        .kind(),
        io::ErrorKind::Unsupported
    );
}
#[test]
fn retained_events_are_bounded() {
    let mut state = state();
    for sequence in 1..=QUEUE {
        state
            .receive(WireMessage::Control(ControlEnvelope {
                stamp: state.stamp,
                sequence: sequence as u64,
                message: Control::Cancel {
                    id: [0; 16],
                    reason: ReplyError::Cancelled,
                },
            }))
            .unwrap();
    }
    assert!(matches!(
        state.receive(WireMessage::Control(ControlEnvelope {
            stamp: state.stamp,
            sequence: 100,
            message: Control::Cancel {
                id: [0; 16],
                reason: ReplyError::Cancelled
            }
        })),
        Err(ProtocolError::Busy)
    ));
}

#[test]
fn escaped_reply_over_wire_cap_does_not_queue_or_close_client() {
    let (tx, rx) = mpsc::sync_channel(QUEUE);
    let client = ChoraClient(Arc::new(Owner {
        diagnostics: Arc::new(Mutex::new(WorkerDiagnostics::default())),
        state: Arc::new(Mutex::new(state())),
        output: tx,
        stop: Arc::new(AtomicBool::new(false)),
    }));
    assert!(matches!(
        client.reply(stamp(), [1; 16], Ok("\0".repeat(MAX_REPLY_BYTES))),
        Err(ProtocolError::TooLarge)
    ));
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    assert!(client.is_ready());
    client
        .reply(stamp(), [1; 16], Err(ReplyError::TooLarge))
        .unwrap();
    assert!(matches!(
        rx.try_recv().unwrap().into_wire(),
        WireMessage::Control(ControlEnvelope {
            message: Control::PageReply {
                result: Err(ReplyError::TooLarge),
                ..
            },
            ..
        })
    ));
}

#[test]
fn full_request_burst_keeps_bounded_lifecycle_slack() {
    let mut state = state();
    let caller = CallerContext {
        engine_instance: [1; 16],
        webview: "test".into(),
        pipeline: "test".into(),
        navigation_epoch: [2; 16],
        origin: "https://fixture.test".into(),
        private_browsing: false,
    };
    for id in 0..MAX_PENDING {
        state
            .receive(WireMessage::Control(ControlEnvelope {
                stamp: state.stamp,
                sequence: id as u64 + 1,
                message: Control::PageRequest {
                    id: [id as u8; 16],
                    caller: caller.clone(),
                    channel: "qualia.graph.snapshot".into(),
                    payload: "{}".into(),
                },
            }))
            .unwrap();
    }
    assert!(matches!(
        state.receive(WireMessage::Control(ControlEnvelope {
            stamp: state.stamp,
            sequence: 65,
            message: Control::PageRequest {
                id: [64; 16],
                caller,
                channel: "qualia.graph.snapshot".into(),
                payload: "{}".into()
            }
        })),
        Err(ProtocolError::Busy)
    ));
    for sequence in 66..74 {
        state
            .receive(WireMessage::Control(ControlEnvelope {
                stamp: state.stamp,
                sequence,
                message: Control::Cancel {
                    id: [0; 16],
                    reason: ReplyError::Cancelled,
                },
            }))
            .unwrap();
    }
    assert_eq!(state.events.len(), QUEUE);
    let (tx, rx) = mpsc::sync_channel(QUEUE);
    let client = ChoraClient(Arc::new(Owner {
        diagnostics: Arc::new(Mutex::new(WorkerDiagnostics::default())),
        state: Arc::new(Mutex::new(state)),
        output: tx,
        stop: Arc::new(AtomicBool::new(false)),
    }));
    for id in 0..MAX_PENDING {
        client
            .reply(stamp(), [id as u8; 16], Err(ReplyError::Denied))
            .unwrap();
    }
    for _ in 0..8 {
        client.cancel(stamp(), [0; 16]).unwrap();
    }
    assert!(matches!(
        client.cancel(stamp(), [0; 16]),
        Err(ProtocolError::Busy)
    ));
    assert!(client.is_ready());
    assert_eq!(rx.try_iter().count(), QUEUE);
}

#[test]
fn diagnostics_keeps_exact_bounded_tail_and_reports_completion() {
    let mut input = vec![b'a'; DIAGNOSTIC_BYTES * 3 + 17];
    let end = input.len();
    input[end - 4..].copy_from_slice(&[0xff, 0, b'Z', b'!']);
    let retained = Mutex::new(WorkerDiagnostics::default());
    drain_diagnostics(&input[..], &retained);
    let tail = retained.lock().unwrap();
    assert_eq!(tail.bytes().len(), DIAGNOSTIC_BYTES);
    assert_eq!(tail.bytes(), &input[input.len() - DIAGNOSTIC_BYTES..]);
    assert!(tail.truncated);
    assert!(tail.complete);
    assert_eq!(tail.read_error, None);
}

#[test]
fn diagnostics_retains_bytes_on_read_error_and_recovers_poison() {
    struct BrokenReader(bool);
    impl io::Read for BrokenReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if !self.0 {
                self.0 = true;
                buffer[..3].copy_from_slice(b"abc");
                Ok(3)
            } else {
                Err(io::ErrorKind::BrokenPipe.into())
            }
        }
    }
    let retained = Arc::new(Mutex::new(WorkerDiagnostics::default()));
    let poison = retained.clone();
    assert!(
        thread::spawn(move || {
            let _guard = poison.lock().unwrap();
            panic!("poison diagnostic lock");
        })
        .join()
        .is_err()
    );
    drain_diagnostics(BrokenReader(false), &retained);
    let (output, _) = mpsc::sync_channel(QUEUE);
    let client = ChoraClient(Arc::new(Owner {
        diagnostics: retained.clone(),
        state: Arc::new(Mutex::new(state())),
        output,
        stop: Arc::new(AtomicBool::new(false)),
    }));
    let tail = client.diagnostics().unwrap();
    assert_eq!(tail.bytes(), b"abc");
    assert_eq!(tail.read_error, Some(io::ErrorKind::BrokenPipe));
    assert!(tail.lock_poisoned);
    assert!(!tail.complete);
    let _guard = retained.lock().unwrap_or_else(|error| error.into_inner());
    assert!(matches!(client.diagnostics(), Err(ProtocolError::Busy)));
}

#[test]
fn input_keeps_route_order_and_coalesces_pointer_moves_in_a_full_queue() {
    let (output, rx) = mpsc::sync_channel(2);
    let client = ChoraClient(Arc::new(Owner {
        diagnostics: Arc::new(Mutex::new(WorkerDiagnostics::default())),
        state: Arc::new(Mutex::new(state())),
        output,
        stop: Arc::new(AtomicBool::new(false)),
    }));
    let stamp = stamp();
    let mut moved = WirePointerEvent {
        action: PointerAction::Move,
        x_px: 7.0,
        y_px: 0.0,
        button: None,
        buttons: 0,
        modifiers: InputModifiers::default(),
    };
    let stale = RouteStamp {
        document: stamp.document + 1,
        ..stamp
    };
    assert!(matches!(
        client.send_pointer(stale, moved),
        Err(ProtocolError::StaleFrame)
    ));
    client.set_focus(stamp, true).unwrap();
    client.send_pointer(stamp, moved).unwrap();
    moved.x_px = 13.0;
    client.send_pointer(stamp, moved).unwrap();
    assert!(matches!(
        client.set_focus(stamp, false),
        Err(ProtocolError::Busy)
    ));
    assert!(matches!(
        rx.try_recv().unwrap().into_wire(),
        WireMessage::Control(ControlEnvelope {
            message: Control::SetFocus { focused: true },
            ..
        })
    ));
    assert!(matches!(rx.try_recv().unwrap().into_wire(),
        WireMessage::Control(ControlEnvelope { stamp: seen, message: Control::InputPointer(event), .. })
            if seen == stamp && event == moved));
    client.set_focus(stamp, false).unwrap();
    assert!(matches!(
        rx.try_recv().unwrap().into_wire(),
        WireMessage::Control(ControlEnvelope {
            message: Control::SetFocus { focused: false },
            ..
        })
    ));
}
