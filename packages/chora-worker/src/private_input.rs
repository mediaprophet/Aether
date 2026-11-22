// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Bounded pipe input with independent cleanup when the parent disappears.
use protocol::{MAX_PENDING, ProtocolError, WireMessage, read_message};
use std::{
    io,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};
fn terminate_worker() -> ! {
    // The deadline must not run Rust/C cleanup or DLL detach callbacks: those
    // can wait on locks held by the stuck engine. Only this process is stopped.
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Threading::{GetCurrentProcess, TerminateProcess};
        // SAFETY: the pseudo-handle names this worker, not another process.
        // Successful self-termination does not return.
        unsafe { TerminateProcess(GetCurrentProcess(), 1) };
        std::process::abort();
    }
    #[cfg(unix)]
    // SAFETY: intentionally bypasses cleanup after loss of this worker's owner.
    unsafe {
        libc::_exit(1)
    }
    #[cfg(not(any(windows, unix)))]
    std::process::abort()
}
fn shutdown_watchdog() {
    // The engine thread may be stuck in GL or engine destruction. This thread
    // owns no engine state and terminates only this worker, without a parent
    // process being required to remain alive to reap an unresponsive engine.
    let watchdog = thread::Builder::new()
        .name("chora-input-deadline".into())
        .spawn(|| {
            thread::sleep(Duration::from_secs(2));
            terminate_worker();
        });
    if watchdog.is_err() {
        terminate_worker();
    }
}
pub fn input_thread() -> Receiver<Result<WireMessage, ProtocolError>> {
    let (tx, rx) = mpsc::sync_channel(MAX_PENDING + 8);
    let engine_thread = thread::current();
    thread::spawn(move || {
        let mut input = io::stdin().lock();
        loop {
            let message = read_message(&mut input).and_then(|message| {
                // Parent controls only: reject pixel frames before retaining any
                // in the input queue, even from a malformed parent transport.
                if matches!(message, WireMessage::Frame(_)) {
                    Err(ProtocolError::WrongDirection)
                } else {
                    Ok(message)
                }
            });
            let failed = message.is_err();
            if failed {
                shutdown_watchdog();
                let _ = tx.try_send(message);
                engine_thread.unpark();
                break;
            }
            // Blocking here could hide parent EOF forever when the engine is
            // stuck and the queue is full. Overload instead terminates safely.
            if tx.try_send(message).is_err() {
                shutdown_watchdog();
                engine_thread.unpark();
                break;
            }
            engine_thread.unpark();
        }
    });
    rx
}
