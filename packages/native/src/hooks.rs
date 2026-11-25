use crate::event_handlers::{WindowEventHandlers, WinitEventHandlerId};

use aether_core::{
    Runtime, consume_context, current_scope_id, try_consume_context, use_hook_with_cleanup,
};
use blitz_dom::BaseDocument;
use std::{cell::RefCell, rc::Rc};
use winit::{
    event::{ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
};

/// Update the active native window's document zoom factor (`0.5..=3.0`).
///
/// Modifying `Viewport::zoom` automatically triggers Stylo media/rem recalculation,
/// Parley inline text re-shaping at the new effective scale, Taffy layout update,
/// and a window redraw.
pub fn set_document_zoom(zoom: f32) -> Option<f32> {
    let clamped = if zoom.is_finite() {
        zoom.clamp(0.5, 3.0)
    } else {
        1.0
    };
    let doc = try_consume_context::<Rc<RefCell<BaseDocument>>>()?;
    if let Ok(mut guard) = doc.try_borrow_mut() {
        if (guard.viewport().zoom - clamped).abs() > 0.001 {
            guard.viewport_mut().zoom = clamped;
        }
        Some(guard.viewport().zoom)
    } else {
        None
    }
}

/// Read the active native window's current document zoom factor.
pub fn document_zoom() -> Option<f32> {
    let doc = try_consume_context::<Rc<RefCell<BaseDocument>>>()?;
    doc.try_borrow().ok().map(|guard| guard.viewport().zoom)
}


/// Register an event handler that runs when a winit event is processed.
///
/// Handlers run in registration order. Removing a handler takes effect immediately,
/// including during dispatch; a new handler starts with the next dispatch. Nested
/// dispatch skips any handler already running so its mutable closure is not reentered.
pub fn use_window_event(
    mut handler: impl FnMut(&WindowEvent, &dyn ActiveEventLoop) + 'static,
) -> WinitEventHandlerId {
    let runtime = Runtime::current();
    let scope_id = current_scope_id();
    let window_id = crate::use_window().id();

    use_hook_with_cleanup(
        move || {
            let handlers: Rc<WindowEventHandlers> = consume_context();
            handlers.add(window_id, move |event, target| {
                runtime.in_scope(scope_id, || handler(event, target))
            })
        },
        move |handler| handler.remove(),
    )
}

/// Register a handler that runs when the back button is pressed.
///
/// This builds on top of [`use_window_event`]: the back button is delivered by `winit` as a
/// [`WindowEvent::KeyboardInput`] whose logical key is [`NamedKey::BrowserBack`]. This most
/// commonly comes from the Android hardware/system back button, but may also be produced by a
/// keyboard or mouse back key on other platforms. The provided `handler` is called once each
/// time the button is pressed (key repeats are ignored).
///
/// Returns a [`WinitEventHandlerId`] which can be used to remove the handler.
pub fn use_back_button(mut handler: impl FnMut() + 'static) -> WinitEventHandlerId {
    use_window_event(move |event, _target| {
        if let WindowEvent::KeyboardInput { event, .. } = event
            && event.state == ElementState::Pressed
            && !event.repeat
            && event.logical_key == Key::Named(NamedKey::BrowserBack)
        {
            handler();
        }
    })
}
