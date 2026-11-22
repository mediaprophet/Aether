// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0
//! Copied Chora frames painted by the native renderer, without linking Chora's
//! engine, Stylo or ICU graph into the native application.
use aether_chora_client::{
    ChoraClient,
    protocol::{
        InputModifiers, PointerAction, PointerButton, ProtocolError, RouteStamp, WireImeEvent,
        WireKeyEvent, WireKeyState, WirePointerEvent, WireWheelEvent,
    },
};
use anyrender::{PaintScene, RenderContext, Scene};
use blitz_dom::{Widget, node::ComputedStyles};
use blitz_traits::events::{
    BlitzImeEvent, BlitzPointerEvent, BlitzWheelDelta, MouseEventButton, UiEvent,
};
use keyboard_types::Modifiers;
use peniko::{ImageBrush, ImageData, ImageFormat, kurbo::Affine};
use std::sync::Arc;

/// A single view's readback image. The host services client events separately
/// through its canonical graph adapter; the widget never authorizes requests.
/// The caller must explicitly select the admitted readback alpha interpretation.
/// Dropping or disconnecting this widget requests bounded background shutdown.
pub struct ChoraFrameWidget {
    client: ChoraClient,
    image: Option<(RouteStamp, ImageBrush)>,
    alpha: ImageAlphaType,
    capture_id: u64,
    disconnected: bool,
    last_scale: f64,
    last_stamp: Option<RouteStamp>,
    focused: bool,
    sent_focus: Option<(RouteStamp, bool)>,
    last_move_px: Option<(i32, i32)>,
}
impl ChoraFrameWidget {
    pub fn new(client: ChoraClient, alpha: ImageAlphaType) -> Self {
        Self {
            client,
            image: None,
            alpha,
            capture_id: 0,
            disconnected: false,
            last_scale: 1.0,
            last_stamp: None,
            focused: false,
            sent_focus: None,
            last_move_px: None,
        }
    }
    fn sync_focus(&mut self) -> bool {
        let Some(stamp) = self.last_stamp else {
            return false;
        };
        let desired = (stamp, self.focused);
        if self.sent_focus == Some(desired) {
            return true;
        }
        if self.client.set_focus(stamp, self.focused).is_ok() {
            self.sent_focus = Some(desired);
            return true;
        }
        false
    }

    fn pointer(&mut self, stamp: RouteStamp, action: PointerAction, event: &BlitzPointerEvent) {
        let x_px = (f64::from(event.coords.client_x) * self.last_scale) as f32;
        let y_px = (f64::from(event.coords.client_y) * self.last_scale) as f32;
        let pixel = (x_px as i32, y_px as i32);
        let is_move = action == PointerAction::Move;
        if is_move && self.last_move_px == Some(pixel) {
            return;
        }
        let button =
            matches!(action, PointerAction::Down | PointerAction::Up).then(|| match event.button {
                MouseEventButton::Main => PointerButton::Main,
                MouseEventButton::Auxiliary => PointerButton::Auxiliary,
                MouseEventButton::Secondary => PointerButton::Secondary,
                MouseEventButton::Fourth => PointerButton::Fourth,
                MouseEventButton::Fifth => PointerButton::Fifth,
            });
        if !is_move {
            self.last_move_px = None;
        }
        if self
            .client
            .send_pointer(
                stamp,
                WirePointerEvent {
                    action,
                    x_px,
                    y_px,
                    button,
                    buttons: event.buttons.bits(),
                    modifiers: input_modifiers(event.mods),
                },
            )
            .is_ok()
            && is_move
        {
            self.last_move_px = Some(pixel);
        }
    }
}
fn input_modifiers(modifiers: Modifiers) -> InputModifiers {
    InputModifiers {
        shift: modifiers.contains(Modifiers::SHIFT),
        ctrl: modifiers.contains(Modifiers::CONTROL),
        alt: modifiers.contains(Modifiers::ALT),
        meta: modifiers.contains(Modifiers::META),
    }
}

impl Widget for ChoraFrameWidget {
    fn focus_changed(&mut self, focused: bool) {
        self.focused = focused;
        self.sync_focus();
    }

    fn handle_event(&mut self, event: &UiEvent) {
        let Some(stamp) = self.last_stamp else {
            return;
        };
        match event {
            UiEvent::PointerMove(event) => self.pointer(stamp, PointerAction::Move, event),
            UiEvent::PointerDown(event) => self.pointer(stamp, PointerAction::Down, event),
            UiEvent::PointerUp(event) => self.pointer(stamp, PointerAction::Up, event),
            UiEvent::PointerCancel(event) => self.pointer(stamp, PointerAction::Cancel, event),
            UiEvent::PointerLeave(event) => self.pointer(stamp, PointerAction::Leave, event),
            UiEvent::Wheel(event) => {
                let (delta_x, delta_y, delta_is_lines) = match event.delta {
                    BlitzWheelDelta::Lines(x, y) => (x, y, true),
                    BlitzWheelDelta::Pixels(x, y) => (x, y, false),
                };
                let _ = self.client.send_wheel(
                    stamp,
                    WireWheelEvent {
                        delta_x,
                        delta_y,
                        delta_is_lines,
                        x_px: (f64::from(event.coords.client_x) * self.last_scale) as f32,
                        y_px: (f64::from(event.coords.client_y) * self.last_scale) as f32,
                    },
                );
            }
            UiEvent::KeyDown(key) | UiEvent::KeyUp(key) if self.focused && self.sync_focus() => {
                let _ = self.client.send_key(
                    stamp,
                    WireKeyEvent {
                        state: if matches!(event, UiEvent::KeyDown(_)) {
                            WireKeyState::Down
                        } else {
                            WireKeyState::Up
                        },
                        key: key.key.to_string(),
                        code: key.code.to_string(),
                        location: key.location as u8,
                        modifiers: input_modifiers(key.modifiers),
                        repeat: key.is_auto_repeating,
                        is_composing: key.is_composing,
                    },
                );
            }
            UiEvent::Ime(event) if self.focused && self.sync_focus() => {
                let event = match event {
                    BlitzImeEvent::Preedit(text, cursor) => WireImeEvent::Preedit {
                        text: text.clone(),
                        cursor_start: cursor.and_then(|(start, _)| u32::try_from(start).ok()),
                        cursor_end: cursor.and_then(|(_, end)| u32::try_from(end).ok()),
                    },
                    BlitzImeEvent::Commit(text) => WireImeEvent::Commit { text: text.clone() },
                    BlitzImeEvent::Disabled => WireImeEvent::Dismiss,
                    BlitzImeEvent::Enabled | BlitzImeEvent::DeleteSurrounding { .. } => return,
                };
                let _ = self.client.send_ime(stamp, event);
            }
            _ => {}
        }
    }

    fn disconnected(&mut self) {
        self.disconnected = true;
        self.image = None;
        self.last_stamp = None;
        self.client.shutdown();
    }
    fn destroy_surfaces(&mut self) {
        if let Some((stamp, true)) = self.sent_focus.take() {
            let _ = self.client.set_focus(stamp, false);
        }
        self.image = None;
        self.last_stamp = None;
        self.last_move_px = None;
    }
    fn requires_redraw(&self) -> bool {
        // A retained image needs one final paint after exit to erase the old
        // scene before continuous redraws stop.
        !self.disconnected && (self.image.is_some() || !self.client.has_exited())
    }
    fn paint(
        &mut self,
        _: &mut dyn RenderContext,
        _: &ComputedStyles,
        width: u32,
        height: u32,
        scale: f64,
    ) -> Scene {
        let previous_stamp = self.last_stamp.take();
        let previous_scale = self.last_scale;
        self.last_scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        let mut scene = Scene::new();
        if self.disconnected
            || self.client.has_exited()
            || matches!(self.client.stamp(), Err(ProtocolError::Closed))
            || width == 0
            || height == 0
        {
            self.image = None;
            return scene;
        }
        // Resize invalidates outstanding captures and image ownership before any
        // replacement pixels are accepted. There is never a cross-engine GPU handle.
        let stamp = match self.client.resize(width, height) {
            Ok(stamp) => stamp,
            Err(ProtocolError::Busy) => return scene,
            Err(_) => {
                self.image = None;
                return scene;
            }
        };
        if self
            .image
            .as_ref()
            .is_some_and(|(owner, _)| *owner != stamp)
        {
            self.image = None;
        }
        if let Some(frame) = self.client.take_frame() {
            if frame.stamp == stamp {
                let image = ImageData {
                    data: peniko::Blob::new(Arc::new(frame.rgba)),
                    format: ImageFormat::Rgba8,
                    alpha_type: self.alpha,
                    width: frame.width,
                    height: frame.height,
                };
                self.image = Some((stamp, ImageBrush::new(image)));
            }
        }
        // begin_capture rejects a second request while its frame is in flight.
        if let Some(next) = self.capture_id.checked_add(1) {
            if self.client.capture(next).is_ok() {
                self.capture_id = next;
            }
        } else {
            self.client.shutdown();
        }
        // Recheck after mailbox consumption in case a worker invalidation raced it.
        if self.client.stamp().ok() != Some(stamp) {
            self.image = None;
        }
        if let Some((stamp, image)) = &self.image {
            scene.draw_image(image.as_ref(), Affine::IDENTITY);
            self.last_stamp = Some(*stamp);
        }
        if previous_stamp != self.last_stamp || previous_scale != self.last_scale {
            self.last_move_px = None;
        }
        self.sync_focus();
        scene
    }
}
impl Drop for ChoraFrameWidget {
    fn drop(&mut self) {
        self.client.shutdown();
    }
}

/// Explicit readback interpretation; runtime alpha qualification is required.
pub use peniko::ImageAlphaType;
