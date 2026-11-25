// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Parent-owned process transport; canonical host authorization remains separate.
//! UI methods only touch bounded state/queues. Drop requests shutdown without joining.
pub use protocol;
use protocol::*;
use std::{
    collections::VecDeque,
    io,
    path::Path,
    process::{Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
    },
    thread,
    time::{Duration, Instant},
};
const QUEUE: usize = MAX_PENDING + 8;
const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);
const DEADLINE: Duration = Duration::from_secs(MAX_DEADLINE_SECS);
const DIAGNOSTIC_BYTES: usize = 16 * 1024;

/// Bounded raw worker stderr. Decode only at the diagnostic presentation boundary.
#[derive(Clone)]
pub struct WorkerDiagnostics {
    bytes: [u8; DIAGNOSTIC_BYTES],
    len: usize,
    pub truncated: bool,
    pub complete: bool,
    pub read_error: Option<io::ErrorKind>,
    pub lock_poisoned: bool,
}
impl Default for WorkerDiagnostics {
    fn default() -> Self {
        Self {
            bytes: [0; DIAGNOSTIC_BYTES],
            len: 0,
            truncated: false,
            complete: false,
            read_error: None,
            lock_poisoned: false,
        }
    }
}
impl WorkerDiagnostics {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
    fn append(&mut self, incoming: &[u8]) {
        if incoming.len() >= DIAGNOSTIC_BYTES {
            self.truncated |= self.len != 0 || incoming.len() > DIAGNOSTIC_BYTES;
            self.bytes
                .copy_from_slice(&incoming[incoming.len() - DIAGNOSTIC_BYTES..]);
            self.len = DIAGNOSTIC_BYTES;
        } else {
            let discarded = (self.len + incoming.len()).saturating_sub(DIAGNOSTIC_BYTES);
            self.truncated |= discarded != 0;
            self.bytes.copy_within(discarded..self.len, 0);
            self.len -= discarded;
            self.bytes[self.len..self.len + incoming.len()].copy_from_slice(incoming);
            self.len += incoming.len();
        }
    }
}
fn drain_diagnostics(mut reader: impl io::Read, retained: &Mutex<WorkerDiagnostics>) {
    let mut buffer = [0; 2048];
    loop {
        let result = reader.read(&mut buffer);
        if matches!(&result, Err(error) if error.kind() == io::ErrorKind::Interrupted) {
            continue;
        }
        let mut tail = retained.lock().unwrap_or_else(|error| {
            let mut tail = error.into_inner();
            tail.lock_poisoned = true;
            tail
        });
        match result {
            Ok(0) => {
                tail.complete = true;
                break;
            }
            Ok(count) => tail.append(&buffer[..count]),
            Err(error) => {
                tail.read_error = Some(error.kind());
                break;
            }
        }
    }
}

struct State {
    stamp: RouteStamp,
    sequence: Sequence,
    mailbox: FrameMailbox,
    events: VecDeque<ControlEnvelope>,
    ready: bool,
    closed: bool,
    exited: bool,
    dimensions: (u32, u32),
    capture_deadline: Option<Instant>,
    startup_deadline: Instant,
    transition_deadline: Option<Instant>,
    pending_move: Option<Arc<Mutex<Option<WirePointerEvent>>>>,
}
impl State {
    fn invalidate(&mut self, stamp: RouteStamp) {
        self.stamp = stamp;
        self.pending_move = None;
        self.mailbox.invalidate(stamp);
        self.capture_deadline = None;
        self.events.clear();
    }
    fn close(&mut self) {
        if self.closed {
            return;
        }
        self.closed = true;
        self.pending_move = None;
        self.ready = false;
        self.mailbox.close();
        self.events.clear();
    }
    fn receive(&mut self, message: WireMessage) -> Result<(), ProtocolError> {
        if self.closed {
            return Err(ProtocolError::Closed);
        }
        let stamp = message.stamp();
        if stamp.view != self.stamp.view {
            return Err(ProtocolError::InvalidMessage);
        }
        match message {
            WireMessage::Frame(frame) => {
                if stamp != self.stamp {
                    return Ok(());
                }
                if (frame.width, frame.height) != self.dimensions {
                    return Err(ProtocolError::InvalidDimensions);
                }
                self.mailbox.publish(frame)?;
                self.capture_deadline = None;
            }
            WireMessage::Control(envelope) => {
                match &envelope.message {
                    Control::Ready {
                        profile: ExecutionProfile::TrustedEmbedding,
                    } if !self.ready && stamp == self.stamp => {
                        self.ready = true;
                    }
                    Control::Invalidate { .. }
                        if stamp.generation == self.stamp.generation
                            && stamp.viewport == self.stamp.viewport
                            && stamp.document >= self.stamp.document =>
                    {
                        if stamp != self.stamp {
                            self.invalidate(stamp);
                        }
                        self.transition_deadline = None;
                    }
                    Control::Stopped | Control::Failed { .. } => {
                        self.close();
                    }
                    Control::PageRequest { .. } | Control::Cancel { .. }
                        if stamp == self.stamp && self.ready => {}
                    // Messages written before a parent transition can still be in the pipe.
                    _ if stamp != self.stamp => return Ok(()),
                    _ => return Err(ProtocolError::InvalidMessage),
                }
                if self.events.len() == QUEUE
                    || (matches!(&envelope.message, Control::PageRequest { .. })
                        && self
                            .events
                            .iter()
                            .filter(|event| matches!(&event.message, Control::PageRequest { .. }))
                            .count()
                            >= MAX_PENDING)
                {
                    return Err(ProtocolError::Busy);
                }
                self.events.push_back(envelope);
            }
        }
        Ok(())
    }
}
struct Outbound {
    wire: WireMessage,
    latest_move: Option<Arc<Mutex<Option<WirePointerEvent>>>>,
}

impl Outbound {
    fn into_wire(mut self) -> WireMessage {
        if let Some(latest) = self.latest_move {
            if let Ok(mut latest) = latest.lock() {
                if let (Some(event), WireMessage::Control(envelope)) =
                    (latest.take(), &mut self.wire)
                {
                    envelope.message = Control::InputPointer(event);
                }
            }
        }
        self.wire
    }
}

/// Nonblocking, observational diagnostics for the current validated route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RouteTelemetry {
    pub stamp: RouteStamp,
    pub last_frame_sequence: Option<u64>,
    pub queued_page_requests: usize,
    pub forwarded_pointer_events: u64,
    pub forwarded_wheel_events: u64,
    pub forwarded_key_events: u64,
    pub forwarded_ime_events: u64,
    pub focus_active: bool,
}

#[derive(Default)]
struct ObservedTransport {
    frame: Option<(RouteStamp, u64)>,
    focus: Option<(RouteStamp, bool)>,
    pointer_events: u64,
    wheel_events: u64,
    key_events: u64,
    ime_events: u64,
}

impl ObservedTransport {
    fn record_write(&mut self, message: &WireMessage) {
        let WireMessage::Control(envelope) = message else {
            return;
        };
        match &envelope.message {
            Control::InputPointer(_) => self.pointer_events = self.pointer_events.saturating_add(1),
            Control::InputWheel(_) => self.wheel_events = self.wheel_events.saturating_add(1),
            Control::InputKey(_) => self.key_events = self.key_events.saturating_add(1),
            Control::InputIme(_) => self.ime_events = self.ime_events.saturating_add(1),
            Control::SetFocus { focused } => self.focus = Some((envelope.stamp, *focused)),
            _ => {}
        }
    }
}

struct Owner {
    state: Arc<Mutex<State>>,
    diagnostics: Arc<Mutex<WorkerDiagnostics>>,
    observed: Arc<Mutex<ObservedTransport>>,
    output: SyncSender<Outbound>,
    stop: Arc<AtomicBool>,
}
impl Drop for Owner {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
    }
}
/// Clones share one process and one latest-frame slot. No worker credentials or
/// authority travel in the environment; session nonces bind only the private pipe.
#[derive(Clone)]
pub struct ChoraClient(Arc<Owner>);
impl ChoraClient {
    /// Caller explicitly opts into a trusted, unsandboxed compatibility process.
    /// Required sandbox and private modes are rejected before process creation.
    pub fn launch(
        executable: &Path,
        profile: ExecutionProfile,
        private_browsing: bool,
    ) -> io::Result<Self> {
        if profile != ExecutionProfile::TrustedEmbedding || private_browsing {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "worker containment/private profile is unavailable",
            ));
        }
        if !executable.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "worker path must be absolute",
            ));
        }
        let mut nonce = [0; 16];
        getrandom::getrandom(&mut nonce)
            .map_err(|_| io::Error::other("worker session entropy unavailable"))?;
        let stamp = RouteStamp {
            session: nonce,
            view: 1,
            generation: 0,
            document: 0,
            viewport: 0,
        };
        let state = Arc::new(Mutex::new(State {
            stamp,
            sequence: Sequence::default(),
            mailbox: FrameMailbox::new(stamp),
            events: VecDeque::new(),
            ready: false,
            closed: false,
            exited: false,
            dimensions: (1, 1),
            capture_deadline: None,
            startup_deadline: Instant::now() + DEADLINE,
            transition_deadline: None,
            pending_move: None,
        }));
        let stop = Arc::new(AtomicBool::new(false));
        let mut command = Command::new(executable);
        command
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for name in [
            "PATH",
            "SystemRoot",
            "WINDIR",
            "TEMP",
            "TMP",
            "DISPLAY",
            "WAYLAND_DISPLAY",
            "XDG_RUNTIME_DIR",
        ] {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000); // CREATE_NO_WINDOW: worker is never an interactive console.
        }
        let mut child = command.spawn()?;
        let mut input = child.stdin.take().expect("piped child stdin");
        let mut output = child.stdout.take().expect("piped child stdout");
        let diagnostics = child.stderr.take().expect("piped child stderr");
        // GUI parents may have no stderr handle. Always give the worker a valid
        // destination for its redirected stdout, and drain without retaining
        // unbounded diagnostics or blocking on the parent's console/log sink.
        let retained_diagnostics = Arc::new(Mutex::new(WorkerDiagnostics::default()));
        let drain_tail = retained_diagnostics.clone();
        thread::spawn(move || drain_diagnostics(diagnostics, &drain_tail));
        let (tx, rx) = mpsc::sync_channel(QUEUE);
        let observed = Arc::new(Mutex::new(ObservedTransport::default()));
        let writer_observed = Arc::clone(&observed);
        let writer_state = state.clone();
        let writer_stop = stop.clone();
        thread::spawn(move || {
            loop {
                match rx.recv_timeout(Duration::from_millis(50)) {
                    Ok(message) => {
                        let message = Outbound::into_wire(message);
                        if write_message(&mut input, &message).is_err() {
                            writer_stop.store(true, Ordering::Release);
                            break;
                        }
                        if let Ok(mut observed) = writer_observed.lock() {
                            observed.record_write(&message);
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if writer_state.lock().map_or(true, |s| s.exited) {
                            break;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });
        let reader_state = state.clone();
        let reader_observed = Arc::clone(&observed);
        let reader_stop = stop.clone();
        thread::spawn(move || {
            let mut guard = PeerGuard::new(PeerRole::Parent, nonce);
            loop {
                let message = read_message(&mut output).and_then(|message| guard.verify(message));
                let Ok(message) = message else {
                    reader_stop.store(true, Ordering::Release);
                    break;
                };
                let Ok(mut state) = reader_state.lock() else {
                    reader_stop.store(true, Ordering::Release);
                    break;
                };
                let message = message.into_inner();
                let frame = match &message {
                    WireMessage::Frame(frame) if frame.stamp == state.stamp => {
                        Some((frame.stamp, frame.sequence))
                    }
                    _ => None,
                };
                if state.receive(message).is_err() {
                    state.close();
                    reader_stop.store(true, Ordering::Release);
                    break;
                }
                if let Some(frame) = frame {
                    if let Ok(mut observed) = reader_observed.lock() {
                        observed.frame = Some(frame);
                    }
                }
                if state.closed {
                    reader_stop.store(true, Ordering::Release);
                    break;
                }
            }
        });
        let monitor_state = state.clone();
        let monitor_stop = stop.clone();
        let monitor_tx = tx.clone();
        thread::spawn(move || {
            let mut stopping = None;
            loop {
                let now = Instant::now();
                let mut state = match monitor_state.lock() {
                    Ok(state) => state,
                    Err(_) => {
                        let _ = child.kill();
                        let _ = child.wait();
                        break;
                    }
                };
                if (!state.ready && now >= state.startup_deadline)
                    || state
                        .capture_deadline
                        .is_some_and(|deadline| now >= deadline)
                    || state
                        .transition_deadline
                        .is_some_and(|deadline| now >= deadline)
                    || state.closed
                {
                    monitor_stop.store(true, Ordering::Release);
                }
                if monitor_stop.load(Ordering::Acquire) && stopping.is_none() {
                    if let Ok(sequence) = state.sequence.next() {
                        let _ = monitor_tx.try_send(Outbound {
                            wire: WireMessage::Control(ControlEnvelope {
                                stamp: state.stamp,
                                sequence,
                                message: Control::Shutdown,
                            }),
                            latest_move: None,
                        });
                    }
                    state.close();
                    stopping = Some(now + SHUTDOWN_GRACE);
                }
                drop(state);
                let done = match child.try_wait() {
                    Ok(Some(_)) => true,
                    Ok(None) => false,
                    Err(_) => {
                        let _ = child.kill();
                        let _ = child.wait();
                        true
                    }
                };
                if done || stopping.is_some_and(|deadline| now >= deadline) {
                    if !done {
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                    if let Ok(mut state) = monitor_state.lock() {
                        state.close();
                        state.exited = true;
                    }
                    break;
                }
                thread::sleep(Duration::from_millis(20));
            }
        });
        let client = Self(Arc::new(Owner {
            state,
            diagnostics: retained_diagnostics,
            observed,
            output: tx,
            stop,
        }));
        client.send_initial().map_err(io::Error::other)?;
        Ok(client)
    }
    fn send_initial(&self) -> Result<(), ProtocolError> {
        let mut state = self.0.state.lock().map_err(|_| ProtocolError::Closed)?;
        let stamp = state.stamp;
        self.send(
            &mut state,
            stamp,
            Control::Initialize {
                profile: ExecutionProfile::TrustedEmbedding,
                private_browsing: false,
            },
        )
    }
    fn send(
        &self,
        state: &mut State,
        stamp: RouteStamp,
        message: Control,
    ) -> Result<(), ProtocolError> {
        if state.closed || self.0.stop.load(Ordering::Acquire) {
            return Err(ProtocolError::Closed);
        }
        let envelope = ControlEnvelope {
            stamp,
            sequence: state.sequence.next()?,
            message,
        };
        envelope.validate()?;
        let wire = WireMessage::Control(envelope);
        // Reply escaping can exceed the wire cap despite its raw string bound.
        // Use the real bounded codec before enqueueing, never failing later in
        // the writer thread and taking down an otherwise healthy worker.
        if matches!(
            &wire,
            WireMessage::Control(ControlEnvelope {
                message: Control::PageReply { .. } | Control::InputIme(_),
                ..
            })
        ) {
            write_message(&mut io::sink(), &wire)?;
        }
        let latest_move = match &wire {
            WireMessage::Control(ControlEnvelope {
                message: Control::InputPointer(event),
                ..
            }) if event.action == PointerAction::Move => {
                if let Some(latest) = &state.pending_move {
                    let mut slot = latest.try_lock().map_err(|_| ProtocolError::Busy)?;
                    if let Some(pending) = slot.as_mut() {
                        *pending = *event;
                        return Ok(());
                    }
                }
                Some(Arc::new(Mutex::new(Some(*event))))
            }
            _ => None,
        };
        self.0
            .output
            .try_send(Outbound {
                wire,
                latest_move: latest_move.clone(),
            })
            .map_err(|error| match error {
                mpsc::TrySendError::Full(_) => ProtocolError::Busy,
                mpsc::TrySendError::Disconnected(_) => ProtocolError::Closed,
            })?;
        state.pending_move = latest_move;
        Ok(())
    }

    fn send_input(&self, expected_stamp: RouteStamp, input: Control) -> Result<(), ProtocolError> {
        let mut state = self.ready_state()?;
        if state.exited {
            return Err(ProtocolError::Closed);
        }
        if state.stamp != expected_stamp {
            return Err(ProtocolError::StaleFrame);
        }
        self.send(&mut state, expected_stamp, input)
    }

    pub fn send_pointer(
        &self,
        expected_stamp: RouteStamp,
        event: WirePointerEvent,
    ) -> Result<(), ProtocolError> {
        self.send_input(expected_stamp, Control::InputPointer(event))
    }

    pub fn send_wheel(
        &self,
        expected_stamp: RouteStamp,
        event: WireWheelEvent,
    ) -> Result<(), ProtocolError> {
        self.send_input(expected_stamp, Control::InputWheel(event))
    }

    pub fn send_key(
        &self,
        expected_stamp: RouteStamp,
        event: WireKeyEvent,
    ) -> Result<(), ProtocolError> {
        self.send_input(expected_stamp, Control::InputKey(event))
    }

    pub fn send_ime(
        &self,
        expected_stamp: RouteStamp,
        event: WireImeEvent,
    ) -> Result<(), ProtocolError> {
        self.send_input(expected_stamp, Control::InputIme(event))
    }

    pub fn set_focus(
        &self,
        expected_stamp: RouteStamp,
        focused: bool,
    ) -> Result<(), ProtocolError> {
        self.send_input(expected_stamp, Control::SetFocus { focused })
    }

    pub fn stamp(&self) -> Result<RouteStamp, ProtocolError> {
        let state = self.0.state.try_lock().map_err(|_| ProtocolError::Busy)?;
        if state.closed || self.0.stop.load(Ordering::Acquire) {
            return Err(ProtocolError::Closed);
        }
        Ok(state.stamp)
    }
    /// Counts input messages successfully written to the worker pipe. Coalesced
    /// moves count once, and closing the transport never exposes stale pixels.
    pub fn route_telemetry(&self) -> Result<RouteTelemetry, ProtocolError> {
        let state = self.0.state.try_lock().map_err(|_| ProtocolError::Busy)?;
        if state.closed || self.0.stop.load(Ordering::Acquire) {
            return Err(ProtocolError::Closed);
        }
        let observed = self
            .0
            .observed
            .try_lock()
            .map_err(|_| ProtocolError::Busy)?;
        let stamp = state.stamp;
        Ok(RouteTelemetry {
            stamp,
            last_frame_sequence: observed
                .frame
                .filter(|(owner, _)| *owner == stamp)
                .map(|(_, sequence)| sequence),
            queued_page_requests: state
                .events
                .iter()
                .filter(|envelope| matches!(envelope.message, Control::PageRequest { .. }))
                .count(),
            forwarded_pointer_events: observed.pointer_events,
            forwarded_wheel_events: observed.wheel_events,
            forwarded_key_events: observed.key_events,
            forwarded_ime_events: observed.ime_events,
            focus_active: observed.focus == Some((stamp, true)),
        })
    }

    pub fn is_ready(&self) -> bool {
        if self.0.stop.load(Ordering::Acquire) {
            return false;
        }
        self.0.state.try_lock().is_ok_and(|s| s.ready && !s.closed)
    }
    pub fn has_exited(&self) -> bool {
        self.0.state.try_lock().is_ok_and(|s| s.exited)
    }
    /// Never waits for the diagnostic drainer or writes to a parent console.
    pub fn diagnostics(&self) -> Result<WorkerDiagnostics, ProtocolError> {
        match self.0.diagnostics.try_lock() {
            Ok(tail) => Ok(tail.clone()),
            Err(std::sync::TryLockError::WouldBlock) => Err(ProtocolError::Busy),
            Err(std::sync::TryLockError::Poisoned(error)) => {
                let mut tail = error.into_inner().clone();
                tail.lock_poisoned = true;
                Ok(tail)
            }
        }
    }
    pub fn take_event(&self) -> Option<ControlEnvelope> {
        self.0.state.try_lock().ok()?.events.pop_front()
    }
    pub fn take_frame(&self) -> Option<Frame> {
        if self.0.stop.load(Ordering::Acquire) {
            return None;
        }
        self.0.state.try_lock().ok()?.mailbox.take()
    }
    fn ready_state(&self) -> Result<std::sync::MutexGuard<'_, State>, ProtocolError> {
        let state = self.0.state.try_lock().map_err(|_| ProtocolError::Busy)?;
        if state.closed || self.0.stop.load(Ordering::Acquire) {
            return Err(ProtocolError::Closed);
        }
        if !state.ready {
            return Err(ProtocolError::Busy);
        }
        Ok(state)
    }
    pub fn navigate(&self, url: String) -> Result<RouteStamp, ProtocolError> {
        let mut state = self.ready_state()?;
        let stamp = RouteStamp {
            generation: state
                .stamp
                .generation
                .checked_add(1)
                .ok_or(ProtocolError::SequenceExhausted)?,
            ..state.stamp
        };
        self.send(&mut state, stamp, Control::Navigate { url })?;
        state.invalidate(stamp);
        state.transition_deadline = Some(Instant::now() + DEADLINE);
        Ok(stamp)
    }
    pub fn resize(&self, width: u32, height: u32) -> Result<RouteStamp, ProtocolError> {
        checked_frame_layout(width, height)?;
        let mut state = self.ready_state()?;
        if state.dimensions == (width, height) {
            return Ok(state.stamp);
        }
        let stamp = RouteStamp {
            viewport: state
                .stamp
                .viewport
                .checked_add(1)
                .ok_or(ProtocolError::SequenceExhausted)?,
            ..state.stamp
        };
        self.send(&mut state, stamp, Control::Resize { width, height })?;
        state.invalidate(stamp);
        state.dimensions = (width, height);
        state.transition_deadline = Some(Instant::now() + DEADLINE);
        Ok(stamp)
    }
    pub fn capture(&self, id: u64) -> Result<(), ProtocolError> {
        let mut state = self.ready_state()?;
        state.mailbox.begin_capture(id)?;
        let stamp = state.stamp;
        if let Err(error) = self.send(&mut state, stamp, Control::Capture { capture_id: id }) {
            state.mailbox.cancel_capture();
            return Err(error);
        }
        state.capture_deadline = Some(Instant::now() + DEADLINE);
        Ok(())
    }
    pub fn reply(
        &self,
        stamp: RouteStamp,
        id: Nonce,
        result: Result<String, ReplyError>,
    ) -> Result<(), ProtocolError> {
        let mut state = self.0.state.try_lock().map_err(|_| ProtocolError::Busy)?;
        if stamp != state.stamp {
            return Err(ProtocolError::StaleFrame);
        }
        self.send(&mut state, stamp, Control::PageReply { id, result })
    }
    pub fn cancel(&self, stamp: RouteStamp, id: Nonce) -> Result<(), ProtocolError> {
        let mut state = self.0.state.try_lock().map_err(|_| ProtocolError::Busy)?;
        if stamp != state.stamp {
            return Err(ProtocolError::StaleFrame);
        }
        self.send(
            &mut state,
            stamp,
            Control::Cancel {
                id,
                reason: ReplyError::Cancelled,
            },
        )
    }
    /// Does not wait. A background owner enforces the shutdown deadline and reaps.
    pub fn shutdown(&self) {
        self.0.stop.store(true, Ordering::Release);
    }
}
#[cfg(test)]
mod tests;
