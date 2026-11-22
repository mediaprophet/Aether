// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
#![cfg(feature = "test-fixture")]
use aether_chora_client::{ChoraClient, protocol::ExecutionProfile};
use std::{
    path::Path,
    thread,
    time::{Duration, Instant},
};
fn until(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(8);
    while !condition() {
        assert!(Instant::now() < deadline, "private-pipe test deadline");
        thread::sleep(Duration::from_millis(10));
    }
}
fn launch() -> ChoraClient {
    let client = ChoraClient::launch(
        Path::new(env!("CARGO_BIN_EXE_transport_fixture")),
        ExecutionProfile::TrustedEmbedding,
        false,
    )
    .unwrap();
    until(|| client.is_ready());
    client
}
#[test]
fn real_private_pipes_deliver_frames_and_reap_uncooperative_child() {
    let client = launch();
    client.resize(2, 3).unwrap();
    until(|| client.capture(1).is_ok());
    let mut frame = None;
    until(|| {
        frame = client.take_frame();
        frame.is_some()
    });
    let frame = frame.unwrap();
    assert_eq!((frame.width, frame.height, frame.rgba.len()), (2, 3, 24));
    let old = frame.stamp;
    client
        .navigate("https://fixture.test/diagnostic-burst".into())
        .unwrap();
    until(|| client.stamp().is_ok_and(|s| s.document > old.document));
    assert!(client.take_frame().is_none());
    let start = Instant::now();
    client.shutdown();
    assert!(start.elapsed() < Duration::from_millis(100));
    until(|| client.has_exited());
    assert!(start.elapsed() < Duration::from_secs(6));
}
#[test]
fn wrong_session_on_private_pipe_closes_and_reaps_worker() {
    let client = launch();
    client
        .navigate("https://fixture.test/wrong-session".into())
        .unwrap();
    until(|| client.has_exited());
    assert!(!client.is_ready());
    assert!(client.take_frame().is_none());
}

#[test]
fn worker_input_watchdog_terminates_when_parent_pipe_disappears() {
    use aether_chora_client::protocol::{
        Control, ControlEnvelope, RouteStamp, WireMessage, read_message, write_message,
    };
    use std::process::{Command, Stdio};
    let mut command = Command::new(env!("CARGO_BIN_EXE_transport_fixture"));
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = child.stdout.take().unwrap();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        let _ = ready_tx.send(read_message(&mut output));
    });
    write_message(
        &mut input,
        &WireMessage::Control(ControlEnvelope {
            stamp: RouteStamp {
                session: [1; 16],
                view: 1,
                generation: 0,
                document: 0,
                viewport: 0,
            },
            sequence: 1,
            message: Control::Initialize {
                profile: ExecutionProfile::TrustedEmbedding,
                private_browsing: false,
            },
        }),
    )
    .unwrap();
    if !matches!(
        ready_rx.recv_timeout(Duration::from_secs(5)),
        Ok(Ok(WireMessage::Control(ControlEnvelope {
            message: Control::Ready { .. },
            ..
        })))
    ) {
        let _ = child.kill();
        let _ = child.wait();
        panic!("worker did not complete its startup handshake");
    }
    // Close stderr only after the peer's startup diagnostic has completed.
    // The watchdog must not attempt diagnostics to this now-broken pipe.
    drop(child.stderr.take());
    write_message(
        &mut input,
        &WireMessage::Control(ControlEnvelope {
            stamp: RouteStamp {
                session: [1; 16],
                view: 1,
                generation: 0,
                document: 0,
                viewport: 0,
            },
            sequence: 2,
            message: Control::Shutdown,
        }),
    )
    .unwrap();
    // The peer's main thread deliberately hangs on Shutdown. No ChoraClient
    // monitor owns this child; only the worker's real EOF watchdog can stop it.
    drop(input);
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(!status.success());
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("worker EOF watchdog missed its deadline");
        }
        thread::sleep(Duration::from_millis(10));
    }
}
