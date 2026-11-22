// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Private-pipe Chora compatibility worker. Process separation is not containment.
use chora::{ChoraBuilder, EmbedderMessageRequest, RenderingContext, WebView, WebViewDelegate};
use protocol::*;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    fs::File,
    rc::Rc,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

mod private_input;
use private_input::input_thread;
mod private_pipe;
use private_pipe::private_output;

struct Pending {
    stamp: RouteStamp,
    deadline: Instant,
    request: EmbedderMessageRequest,
}
struct Delegate {
    stamp: Cell<RouteStamp>,
    sequence: Rc<RefCell<Sequence>>,
    output: Rc<RefCell<File>>,
    active: Cell<bool>,
    pending: RefCell<HashMap<Nonce, Pending>>,
    capture: Cell<Option<u64>>,
    frame_available: Cell<bool>,
    failed: Cell<bool>,
}
impl Delegate {
    fn new(
        stamp: RouteStamp,
        output: Rc<RefCell<File>>,
        sequence: Rc<RefCell<Sequence>>,
    ) -> Rc<Self> {
        Rc::new(Self {
            stamp: Cell::new(stamp),
            output,
            sequence,
            active: Cell::new(true),
            pending: RefCell::new(HashMap::new()),
            capture: Cell::new(None),
            frame_available: Cell::new(false),
            failed: Cell::new(false),
        })
    }
    fn retire(&self) {
        self.active.set(false);
        self.frame_available.set(false);
        self.cancel_all(ReplyError::Cancelled);
    }
    fn send(&self, message: Control) {
        let result = self.sequence.borrow_mut().next().and_then(|sequence| {
            write_message(
                &mut *self.output.borrow_mut(),
                &WireMessage::Control(ControlEnvelope {
                    stamp: self.stamp.get(),
                    sequence,
                    message,
                }),
            )
        });
        if result.is_err() {
            self.failed.set(true);
        }
    }
    fn cancel_all(&self, reason: ReplyError) {
        self.capture.set(None);
        let requests = std::mem::take(&mut *self.pending.borrow_mut());
        for (_, pending) in requests {
            pending.request.respond(Err(engine_error(reason)));
        }
    }
    fn expire(&self) {
        let now = Instant::now();
        let ids: Vec<_> = self
            .pending
            .borrow()
            .iter()
            .filter(|(_, p)| p.deadline <= now || p.request.is_cancelled())
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            let pending = self.pending.borrow_mut().remove(&id);
            if let Some(pending) = pending {
                let reason = if pending.request.is_cancelled() {
                    ReplyError::Cancelled
                } else {
                    ReplyError::TimedOut
                };
                pending.request.respond(Err(engine_error(reason)));
                self.send(Control::Cancel { id, reason });
            }
        }
    }
}
impl WebViewDelegate for Delegate {
    fn notify_closed(&self, _: WebView) {
        if !self.active.get() {
            return;
        }
        self.retire();
        self.failed.set(true);
        self.send(Control::Stopped);
    }
    fn notify_crashed(&self, _: WebView, _: String, _: Option<String>) {
        if !self.active.get() {
            return;
        }
        self.retire();
        self.failed.set(true);
        self.send(Control::Failed {
            reason: WorkerFailure::Rendering,
        });
    }
    fn notify_new_frame_ready(&self, _: WebView) {
        if !self.active.get() {
            return;
        }
        self.frame_available.set(true);
    }
    fn notify_url_changed(&self, _: WebView, _: url::Url) {
        if !self.active.get() {
            return;
        }
        self.cancel_all(ReplyError::Cancelled);
        let mut stamp = self.stamp.get();
        let Some(document) = stamp.document.checked_add(1) else {
            self.failed.set(true);
            return;
        };
        stamp.document = document;
        self.stamp.set(stamp);
        self.frame_available.set(false);
        self.send(Control::Invalidate {
            reason: ReplyError::Cancelled,
        });
    }
    fn notify_embedder_message(&self, _: WebView, request: EmbedderMessageRequest) {
        if !self.active.get() {
            request.cancel();
            return;
        }
        if self.pending.borrow().len() >= MAX_PENDING {
            request.respond(Err(chora::ChoraMessageError::Busy));
            return;
        }
        let id = *request.id().as_bytes();
        if self.pending.borrow().contains_key(&id) {
            request.respond(Err(chora::ChoraMessageError::InvalidContext));
            return;
        }
        let caller = request.caller();
        let context = CallerContext {
            engine_instance: *caller.engine_instance().as_bytes(),
            webview: format!("{:?}", caller.webview_id()),
            pipeline: format!("{:?}", caller.pipeline_id()),
            navigation_epoch: *caller.navigation_epoch().as_bytes(),
            origin: caller.origin().ascii_serialization().into_owned(),
            private_browsing: caller.is_private_browsing(),
        };
        let message = Control::PageRequest {
            id,
            caller: context,
            channel: request.channel.clone(),
            payload: request.payload.clone(),
        };
        let stamp = self.stamp.get();
        self.pending.borrow_mut().insert(
            id,
            Pending {
                stamp,
                deadline: Instant::now() + Duration::from_secs(MAX_DEADLINE_SECS),
                request,
            },
        );
        self.send(message);
    }
    fn notify_embedder_message_cancelled(
        &self,
        _: WebView,
        id: chora::ChoraMessageId,
        _: chora::ChoraMessageError,
    ) {
        if !self.active.get() {
            return;
        }
        let id = *id.as_bytes();
        let pending = self.pending.borrow_mut().remove(&id);
        if let Some(pending) = pending {
            pending.request.cancel();
            self.send(Control::Cancel {
                id,
                reason: ReplyError::Cancelled,
            });
        }
    }
}
fn engine_error(error: ReplyError) -> chora::ChoraMessageError {
    match error {
        ReplyError::Denied => chora::ChoraMessageError::Denied,
        ReplyError::Cancelled => chora::ChoraMessageError::Cancelled,
        ReplyError::TimedOut => chora::ChoraMessageError::TimedOut,
        ReplyError::Unavailable => chora::ChoraMessageError::Unavailable,
        ReplyError::TooLarge => chora::ChoraMessageError::TooLarge,
        ReplyError::InvalidContext => chora::ChoraMessageError::InvalidContext,
        ReplyError::Busy => chora::ChoraMessageError::Busy,
    }
}
#[derive(Clone)]
struct Waker(thread::Thread);
impl chora::EventLoopWaker for Waker {
    fn clone_box(&self) -> Box<dyn chora::EventLoopWaker> {
        Box::new(self.clone())
    }
    fn wake(&self) {
        self.0.unpark();
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let output = private_output()?;
    let input = input_thread();
    let first = input.recv()??;
    let stamp = first.stamp();
    let mut guard = PeerGuard::new(PeerRole::Worker, stamp.session);
    let WireMessage::Control(first) = guard.verify(first)?.into_inner() else {
        return Err(ProtocolError::WrongDirection.into());
    };
    let Control::Initialize {
        profile,
        private_browsing,
    } = first.message
    else {
        return Err(ProtocolError::InvalidMessage.into());
    };
    if stamp.view == 0 || stamp.generation != 0 || stamp.document != 0 || stamp.viewport != 0 {
        return Err(ProtocolError::InvalidMessage.into());
    }
    let output = Rc::new(RefCell::new(output));
    let sequence = Rc::new(RefCell::new(Sequence::default()));
    let mut delegate = Delegate::new(stamp, output.clone(), sequence.clone());
    // No qualified OS containment bootstrap exists for this initial worker.
    // Reject before creating engine globals, rendering contexts or browser threads.
    if profile != ExecutionProfile::TrustedEmbedding || private_browsing {
        delegate.send(Control::Failed {
            reason: if private_browsing {
                WorkerFailure::Initialization
            } else {
                WorkerFailure::UnsupportedSandbox
            },
        });
        return Ok(());
    }
    let context: Rc<dyn RenderingContext> =
        match chora::SoftwareRenderingContext::new(dpi::PhysicalSize::new(1, 1)) {
            Ok(context) => Rc::new(context),
            Err(_) => {
                delegate.send(Control::Failed {
                    reason: WorkerFailure::Rendering,
                });
                return Ok(());
            }
        };
    if context.make_current().is_err() {
        delegate.send(Control::Failed {
            reason: WorkerFailure::Rendering,
        });
        return Ok(());
    }
    let engine = match ChoraBuilder::default()
        .event_loop_waker(Box::new(Waker(thread::current())))
        .try_build()
    {
        Ok(engine) => engine,
        Err(_) => {
            delegate.send(Control::Failed {
                reason: WorkerFailure::Initialization,
            });
            return Ok(());
        }
    };
    let mut view = chora::WebViewBuilder::new(&engine, Rc::clone(&context))
        .delegate(delegate.clone())
        .url(url::Url::parse("about:blank")?)
        .build();
    view.show();
    delegate.send(Control::Ready { profile });
    let mut dimensions = (1, 1);
    let mut browser_input = BrowserInput::default();
    let mut browser_focused = false;
    'running: loop {
        for _ in 0..8 {
            let message = match input.try_recv() {
                Ok(message) => message?,
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => break 'running,
            };
            let WireMessage::Control(envelope) = guard.verify(message)?.into_inner() else {
                return Err(ProtocolError::WrongDirection.into());
            };
            let current = delegate.stamp.get();
            let next = envelope.stamp;
            if next.view != current.view {
                return Err(ProtocolError::InvalidMessage.into());
            }
            match envelope.message {
                Control::Shutdown => break 'running,
                Control::Navigate { url }
                    if next.generation
                        == current
                            .generation
                            .checked_add(1)
                            .ok_or(ProtocolError::SequenceExhausted)?
                        && next.viewport == current.viewport
                        && next.document <= current.document =>
                {
                    let url = url::Url::parse(&url)?;
                    if !matches!(url.scheme(), "https" | "http" | "about") {
                        return Err(ProtocolError::InvalidMessage.into());
                    }
                    // A load request does not commit a document. Retire the entire
                    // old view/delegate before creating the next generation: queued
                    // old callbacks can never acquire the new owner's stamp.
                    delegate.retire();
                    browser_input = BrowserInput::default();
                    browser_focused = false;
                    drop(view);
                    delegate = Delegate::new(
                        RouteStamp {
                            document: current.document,
                            ..next
                        },
                        output.clone(),
                        sequence.clone(),
                    );
                    view = chora::WebViewBuilder::new(&engine, Rc::clone(&context))
                        .delegate(delegate.clone())
                        .url(url)
                        .build();
                    view.show();
                }
                Control::Resize { width, height }
                    if next.generation == current.generation
                        && next.viewport
                            == current
                                .viewport
                                .checked_add(1)
                                .ok_or(ProtocolError::SequenceExhausted)?
                        && next.document <= current.document =>
                {
                    checked_frame_layout(width, height)?;
                    delegate.cancel_all(ReplyError::Cancelled);
                    delegate.stamp.set(RouteStamp {
                        document: current.document,
                        ..next
                    });
                    dimensions = (width, height);
                    delegate.frame_available.set(false);
                    let requested = dpi::PhysicalSize::new(width, height);
                    view.resize(requested);
                    if context.size() != requested {
                        delegate.send(Control::Failed {
                            reason: WorkerFailure::Rendering,
                        });
                        break 'running;
                    }
                    delegate.send(Control::Invalidate {
                        reason: ReplyError::Cancelled,
                    });
                }
                Control::Capture { capture_id } if next == current => {
                    if delegate.capture.replace(Some(capture_id)).is_some() {
                        return Err(ProtocolError::Busy.into());
                    }
                }
                Control::PageReply { id, result } if next == current => {
                    let pending = delegate.pending.borrow_mut().remove(&id);
                    if let Some(pending) = pending {
                        if pending.stamp == current && pending.deadline > Instant::now() {
                            pending.request.respond(result.map_err(engine_error));
                        } else {
                            pending.request.cancel();
                        }
                    }
                }
                Control::Cancel { id, reason } if next == current => {
                    let pending = delegate.pending.borrow_mut().remove(&id);
                    if let Some(pending) = pending {
                        pending.request.respond(Err(engine_error(reason)));
                    }
                }
                Control::InputPointer(event) if next == current => {
                    browser_input.pointer(&view, event)
                }
                Control::InputWheel(event) if next == current => browser_wheel(&view, event),
                Control::InputKey(event) if next == current && browser_focused => {
                    browser_keyboard(&view, event)
                }
                Control::InputIme(event) if next == current && browser_focused => {
                    browser_input.ime(&view, event)
                }
                Control::SetFocus { focused } if next == current => {
                    if !focused && browser_input.composing {
                        browser_input.ime(&view, WireImeEvent::Dismiss);
                    }
                    browser_focused = focused;
                    view.set_focused(focused);
                }
                Control::InputPointer(_)
                | Control::InputWheel(_)
                | Control::InputKey(_)
                | Control::InputIme(_)
                | Control::SetFocus { .. } => {}
                Control::Capture { .. } | Control::PageReply { .. } | Control::Cancel { .. } => {}
                _ => return Err(ProtocolError::InvalidMessage.into()),
            }
        }
        engine.spin_event_loop();
        delegate.expire();
        if delegate.failed.get() {
            break;
        }
        if delegate.frame_available.get()
            && let Some(capture_id) = delegate.capture.get()
        {
            let (width, height) = dimensions;
            let layout = checked_frame_layout(width, height)?;
            if layout.readback_reservation_bytes > MAX_FRAME_BYTES * 4 {
                return Err(ProtocolError::TooLarge.into());
            }
            view.paint();
            let rectangle = chora::DeviceIntRect::from_origin_and_size(
                chora::DeviceIntPoint::zero(),
                chora::DeviceIntSize::new(width as i32, height as i32),
            );
            let image = context
                .read_to_image(rectangle)
                .ok_or(ProtocolError::InvalidDimensions)?;
            let rgba = image.into_raw();
            if rgba.len() != layout.rgba_bytes {
                return Err(ProtocolError::InvalidDimensions.into());
            }
            let frame = Frame {
                stamp: delegate.stamp.get(),
                sequence: delegate.sequence.borrow_mut().next()?,
                capture_id,
                width,
                height,
                rgba,
            };
            // Capture stays occupied until the complete binary frame is written.
            write_message(
                &mut *delegate.output.borrow_mut(),
                &WireMessage::Frame(frame),
            )?;
            delegate.capture.set(None);
            context.present();
        }
        thread::park_timeout(Duration::from_millis(10));
    }
    delegate.retire();
    drop(view);
    // Engine destruction may spin; independent parent deadline cleanup is required.
    drop(engine);
    delegate.send(Control::Stopped);
    Ok(())
}
#[derive(Default)]
struct BrowserInput {
    buttons: [bool; 5],
    composing: bool,
}

impl BrowserInput {
    fn pointer(&mut self, view: &WebView, event: WirePointerEvent) {
        use chora::{InputEvent, MouseButton, MouseButtonAction};
        let point = chora::DevicePoint::new(event.x_px, event.y_px).into();
        let button = |index| match index {
            0 => MouseButton::Primary,
            1 => MouseButton::Auxiliary,
            2 => MouseButton::Secondary,
            3 => MouseButton::Back,
            _ => MouseButton::Forward,
        };
        match event.action {
            PointerAction::Move => {
                view.notify_input_event(InputEvent::MouseMove(chora::MouseMoveEvent::new(point)));
            }
            PointerAction::Down | PointerAction::Up => {
                let Some(which) = event.button else {
                    return;
                };
                let index = match which {
                    PointerButton::Main => 0,
                    PointerButton::Auxiliary => 1,
                    PointerButton::Secondary => 2,
                    PointerButton::Fourth => 3,
                    PointerButton::Fifth => 4,
                };
                let down = event.action == PointerAction::Down;
                self.buttons[index] = down;
                let action = if down {
                    MouseButtonAction::Down
                } else {
                    MouseButtonAction::Up
                };
                view.notify_input_event(InputEvent::MouseButton(chora::MouseButtonEvent::new(
                    action,
                    button(index),
                    point,
                )));
            }
            PointerAction::Cancel | PointerAction::Leave => {
                if event.action == PointerAction::Cancel {
                    for (index, pressed) in self.buttons.iter_mut().enumerate() {
                        if std::mem::take(pressed) {
                            view.notify_input_event(InputEvent::MouseButton(
                                chora::MouseButtonEvent::new(
                                    MouseButtonAction::Up,
                                    button(index),
                                    point,
                                ),
                            ));
                        }
                    }
                }
                view.notify_input_event(InputEvent::MouseLeftViewport(Default::default()));
            }
        }
    }

    fn ime(&mut self, view: &WebView, event: WireImeEvent) {
        use chora::{CompositionEvent, CompositionState, ImeEvent, InputEvent};
        let emit = |state, data| {
            view.notify_input_event(InputEvent::Ime(ImeEvent::Composition(CompositionEvent {
                state,
                data,
            })));
        };
        match event {
            WireImeEvent::Preedit { text, .. } | WireImeEvent::Commit { text }
                if text.is_empty() && !self.composing => {}
            WireImeEvent::Preedit { text, .. } => {
                if !std::mem::replace(&mut self.composing, true) {
                    emit(CompositionState::Start, String::new());
                }
                emit(CompositionState::Update, text);
            }
            WireImeEvent::Commit { text } => {
                if !std::mem::take(&mut self.composing) {
                    emit(CompositionState::Start, String::new());
                }
                emit(CompositionState::Update, text.clone());
                emit(CompositionState::End, text);
            }
            WireImeEvent::Dismiss => {
                self.composing = false;
                view.notify_input_event(InputEvent::Ime(ImeEvent::Dismissed));
            }
        }
    }
}

fn browser_keyboard(view: &WebView, event: WireKeyEvent) {
    use chora::{KeyState, Location, Modifiers};
    let mut modifiers = Modifiers::empty();
    modifiers.set(Modifiers::SHIFT, event.modifiers.shift);
    modifiers.set(Modifiers::CONTROL, event.modifiers.ctrl);
    modifiers.set(Modifiers::ALT, event.modifiers.alt);
    modifiers.set(Modifiers::META, event.modifiers.meta);
    let state = match event.state {
        WireKeyState::Down => KeyState::Down,
        WireKeyState::Up => KeyState::Up,
    };
    let location = match event.location {
        1 => Location::Left,
        2 => Location::Right,
        3 => Location::Numpad,
        _ => Location::Standard,
    };
    view.notify_input_event(chora::InputEvent::Keyboard(
        chora::KeyboardEvent::new_without_event(
            state,
            event.key.parse().unwrap_or_default(),
            event.code.parse().unwrap_or_default(),
            location,
            modifiers,
            event.repeat,
            event.is_composing,
        ),
    ));
}

fn browser_wheel(view: &WebView, event: WireWheelEvent) {
    let mode = if event.delta_is_lines {
        chora::WheelMode::DeltaLine
    } else {
        chora::WheelMode::DeltaPixel
    };
    let delta = chora::WheelDelta {
        x: event.delta_x,
        y: event.delta_y,
        z: 0.0,
        mode,
    };
    let point = chora::DevicePoint::new(event.x_px, event.y_px).into();
    view.notify_input_event(chora::InputEvent::Wheel(chora::WheelEvent::new(
        delta, point,
    )));
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Chora worker stopped: {error}");
        std::process::exit(1);
    }
}
