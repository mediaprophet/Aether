mod driver;
mod focus;
mod ime;
mod keyboard;
mod pointer;

use crate::util::Point;
use blitz_traits::events::{DomEvent, DomEventData, PointerCoords, UiEvent};
pub use driver::{EventDriver, EventHandler, NoopEventHandler};
use focus::generate_focus_events;
pub(crate) use ime::handle_ime_event;
use keyboard::{KeyboardOrTextInputEvent, handle_key_or_input_event};
pub(crate) use pointer::DragMode;
use pointer::{handle_click, handle_pointerdown, handle_pointermove, handle_pointerup};

use crate::{BaseDocument, events::pointer::handle_wheel};

fn adjust_coords_for_subdocument(
    coords: &mut PointerCoords,
    offset: Point<f32>,
    viewport_scroll: Point<f64>,
) {
    // The node's origin and the incoming page coordinates use the same
    // document space. Client coordinates in the embedded view start there.
    coords.client_x = coords.page_x - offset.x;
    coords.client_y = coords.page_y - offset.y;
    coords.page_x = coords.client_x + viewport_scroll.x as f32;
    coords.page_y = coords.client_y + viewport_scroll.y as f32;
}

fn map_dom_event_to_ui_event(
    event: &mut DomEvent,
    node_offset: Point<f32>,
    viewport_scroll: Point<f64>,
) -> Option<UiEvent> {
    // TODO: eliminate clone
    match event.data.clone() {
        DomEventData::PointerMove(mut event) => {
            adjust_coords_for_subdocument(&mut event.coords, node_offset, viewport_scroll);
            Some(UiEvent::PointerMove(event))
        }
        DomEventData::PointerDown(mut event) => {
            adjust_coords_for_subdocument(&mut event.coords, node_offset, viewport_scroll);
            Some(UiEvent::PointerDown(event))
        }
        DomEventData::PointerUp(mut event) => {
            adjust_coords_for_subdocument(&mut event.coords, node_offset, viewport_scroll);
            Some(UiEvent::PointerUp(event))
        }
        DomEventData::PointerCancel(mut event) => {
            adjust_coords_for_subdocument(&mut event.coords, node_offset, viewport_scroll);
            Some(UiEvent::PointerCancel(event))
        }

        // The sub-document receives no further move events once the pointer
        // has left it, so the leave must be forwarded for it to end its hover
        DomEventData::PointerLeave(mut event) => {
            adjust_coords_for_subdocument(&mut event.coords, node_offset, viewport_scroll);
            Some(UiEvent::PointerLeave(event))
        }

        // Enter/over/out events will be recreated by sub-document's event driver
        // based move events
        DomEventData::PointerEnter(_) => None,
        DomEventData::PointerOver(_) => None,
        DomEventData::PointerOut(_) => None,

        // Mouse events will be recreated by sub-document's event driver
        // based pointer events
        DomEventData::MouseMove(_) => None,
        DomEventData::MouseDown(_) => None,
        DomEventData::MouseUp(_) => None,
        DomEventData::MouseEnter(_) => None,
        DomEventData::MouseLeave(_) => None,
        DomEventData::MouseOver(_) => None,
        DomEventData::MouseOut(_) => None,

        // Touch events will be recreated by sub-document's event driver
        // based pointer events
        DomEventData::TouchStart(_) => None,
        DomEventData::TouchMove(_) => None,
        DomEventData::TouchEnd(_) => None,
        DomEventData::TouchCancel(_) => None,

        DomEventData::KeyDown(data) => Some(UiEvent::KeyDown(data)),
        DomEventData::KeyUp(data) => Some(UiEvent::KeyUp(data)),
        DomEventData::Ime(data) => Some(UiEvent::Ime(data)),
        DomEventData::AppleStandardKeybinding(data) => Some(UiEvent::AppleStandardKeybinding(data)),

        DomEventData::KeyPress(_) => None,
        DomEventData::Click(_) => None,
        DomEventData::ContextMenu(_) => None,
        DomEventData::DoubleClick(_) => None,
        DomEventData::Input(_) => None,
        DomEventData::Wheel(mut data) => {
            adjust_coords_for_subdocument(&mut data.coords, node_offset, viewport_scroll);
            Some(UiEvent::Wheel(data))
        }
        DomEventData::Scroll(_) => None,
        DomEventData::Focus(_) => None,
        DomEventData::Blur(_) => None,
        DomEventData::FocusIn(_) => None,
        DomEventData::FocusOut(_) => None,
        DomEventData::Copy(_)
        | DomEventData::Cut(_)
        | DomEventData::Paste(_)
        | DomEventData::BeforeInput(_)
        | DomEventData::Select(_)
        | DomEventData::SelectionChange(_)
        | DomEventData::BeforeToggle(_)
        | DomEventData::Toggle(_)
        | DomEventData::Resize(_)
        | DomEventData::Load(_)
        | DomEventData::Error(_)
        | DomEventData::Change(_)
        | DomEventData::Submit(_)
        | DomEventData::Reset(_)
        | DomEventData::Cancel(_) => None,
    }
}

pub(crate) fn handle_dom_event<F: FnMut(DomEvent)>(
    doc: &mut BaseDocument,
    event: &mut DomEvent,
    mut dispatch_event: F,
) {
    let target_node_id = event.target;
    let Some(node) = doc.get_node_mut(target_node_id) else {
        return;
    };
    let pos = node.absolute_position(0.0, 0.0);

    // Whether this event can move the caret/selection (or change the text) of a text input,
    // in which case we need to update the input's scroll offset afterwards.
    let may_move_text_input_caret = match &event.data {
        DomEventData::KeyDown(_)
        | DomEventData::AppleStandardKeybinding(_)
        | DomEventData::Ime(_)
        | DomEventData::Click(_) => true,
        DomEventData::PointerDown(event) | DomEventData::PointerMove(event) => {
            !event.buttons.is_empty()
        }
        _ => false,
    };

    // Handle event forwarding for sub-document
    if let Some(sub_doc) = node.subdoc_mut() {
        let viewport_scroll = sub_doc.inner().viewport_scroll();

        let set_focus = matches!(
            &event.data,
            DomEventData::PointerDown(_) | DomEventData::PointerUp(_)
        );
        let ui_event = map_dom_event_to_ui_event(event, pos, viewport_scroll);

        if let Some(ui_event) = ui_event {
            sub_doc.handle_ui_event(ui_event);
        }

        if set_focus {
            generate_focus_events(
                doc,
                &mut |doc| {
                    doc.set_focus_to(target_node_id);
                },
                &mut dispatch_event,
            );
        }

        return;
    }

    // Keep widget focus synchronized before forwarding the activating pointer event.
    #[cfg(feature = "custom-widget")]
    if node
        .element_data()
        .is_some_and(|el| el.custom_widget_data().is_some())
    {
        if matches!(
            event.data,
            DomEventData::PointerDown(_) | DomEventData::PointerUp(_)
        ) {
            generate_focus_events(
                doc,
                &mut |doc| {
                    doc.set_focus_to(target_node_id);
                },
                &mut dispatch_event,
            );
        }
        if let Some(event) = map_dom_event_to_ui_event(event, pos, Point { x: 0.0, y: 0.0 }) {
            if let Some(widget) = doc
                .get_node_mut(target_node_id)
                .and_then(|node| node.element_data_mut())
                .and_then(|el| el.custom_widget_data_mut())
            {
                widget.widget.handle_event(&event);
            }
        }
        return;
    }

    match &event.data {
        DomEventData::PointerMove(event) => {
            let changed = handle_pointermove(doc, target_node_id, event, dispatch_event);
            if changed {
                doc.shell_provider.request_redraw();
            }
        }
        DomEventData::MouseMove(_) => {
            // Do nothing (handled in PointerMove)
        }
        DomEventData::PointerDown(event) => {
            handle_pointerdown(
                doc,
                target_node_id,
                event.page_x(),
                event.page_y(),
                event.button,
                event.mods,
                &mut dispatch_event,
            );
        }
        DomEventData::MouseDown(_) => {
            // Do nothing (handled in PointerDown)
        }
        DomEventData::PointerUp(event) => {
            handle_pointerup(doc, target_node_id, event, dispatch_event);
        }
        DomEventData::MouseUp(_) => {
            // Do nothing (handled in PointerUp)
        }
        DomEventData::PointerCancel(_) => {
            // Do nothing (active state is reset in the event driver)
        }
        DomEventData::Click(event) => {
            handle_click(doc, target_node_id, event, &mut dispatch_event);
        }
        DomEventData::KeyDown(event) => {
            handle_key_or_input_event(
                doc,
                target_node_id,
                KeyboardOrTextInputEvent::KeyPress(event.clone()),
                dispatch_event,
            );
        }
        DomEventData::KeyPress(_) => {
            // Do nothing (no default action)
        }
        DomEventData::KeyUp(_) => {
            // Do nothing (no default action)
        }
        DomEventData::AppleStandardKeybinding(event) => {
            handle_key_or_input_event(
                doc,
                target_node_id,
                KeyboardOrTextInputEvent::AppleStandardKeyBinding(event.clone()),
                dispatch_event,
            );
        }
        DomEventData::Ime(event) => {
            handle_ime_event(doc, event.clone(), dispatch_event);
        }
        DomEventData::Input(_) => {
            // Do nothing (no default action)
        }
        DomEventData::ContextMenu(_) => {
            // TODO: Open context menu
        }
        DomEventData::DoubleClick(_) => {
            // Do nothing (no default action)
        }
        DomEventData::PointerEnter(_) => {
            // Do nothing (no default action)
        }
        DomEventData::PointerLeave(_) => {
            // Do nothing (no default action)
        }
        DomEventData::PointerOver(_) => {
            // Do nothing (no default action)
        }
        DomEventData::PointerOut(_) => {
            // Do nothing (no default action)
        }
        DomEventData::MouseEnter(_) => {
            // Do nothing (no default action)
        }
        DomEventData::MouseLeave(_) => {
            // Do nothing (no default action)
        }
        DomEventData::MouseOver(_) => {
            // Do nothing (no default action)
        }
        DomEventData::MouseOut(_) => {
            // Do nothing (no default action)
        }
        DomEventData::TouchStart(_) => {
            // Do nothing (default action handled via PointerDown)
        }
        DomEventData::TouchMove(_) => {
            // Do nothing (default action handled via PointerMove)
        }
        DomEventData::TouchEnd(_) => {
            // Do nothing (default action handled via PointerUp)
        }
        DomEventData::TouchCancel(_) => {
            // Do nothing (default action handled via PointerCancel)
        }
        DomEventData::Scroll(_) => {
            // Handled elsewhere
        }
        DomEventData::Wheel(event) => {
            handle_wheel(doc, target_node_id, event.clone(), dispatch_event);
        }
        DomEventData::Focus(_) => {
            // Do nothing (no default action)
        }
        DomEventData::Blur(_) => {
            // Do nothing (no default action)
        }
        DomEventData::FocusIn(_) => {
            // Do nothing (no default action)
        }
        DomEventData::FocusOut(_) => {
            // Do nothing (no default action)
        }
        DomEventData::Copy(_)
        | DomEventData::Cut(_)
        | DomEventData::Paste(_)
        | DomEventData::BeforeInput(_)
        | DomEventData::Select(_)
        | DomEventData::SelectionChange(_)
        | DomEventData::Toggle(_)
        | DomEventData::Resize(_)
        | DomEventData::Load(_)
        | DomEventData::Error(_)
        | DomEventData::Change(_) => {}
        DomEventData::BeforeToggle(toggle) => {
            if let Some(event) = doc.apply_details_toggle(target_node_id, toggle) {
                dispatch_event(event);
            }
        }
        DomEventData::Submit(_) => {
            let submitter = event
                .related_target
                .filter(|id| {
                    doc.get_node(*id).is_some()
                        && doc.controls_to_form.get(id) == Some(&target_node_id)
                })
                .unwrap_or(target_node_id);
            doc.submit_form(target_node_id, submitter);
        }
        DomEventData::Reset(_) => doc.reset_native_form(target_node_id),
        DomEventData::Cancel(_) => {
            if doc.nodes[target_node_id]
                .data
                .is_element_with_tag_name(&markup5ever::local_name!("dialog"))
            {
                doc.mutate()
                    .clear_attribute(target_node_id, crate::qual_name!("open"));
            }
        }
    }

    // Keep the focused text input scrolled so that its caret stays visible. Keyboard/IME events
    // target the focused input, and pointer events that hit a text input focus it, so the
    // focused node is the input whose caret may have moved.
    if may_move_text_input_caret {
        if let Some(focus_id) = doc.focus_node_id {
            doc.clamp_text_input_scroll(focus_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_input_uses_the_parent_page_origin_with_independent_scroll() {
        let mut coords = PointerCoords {
            page_x: 102.0,
            page_y: 215.0,
            client_x: 42.0,
            client_y: 35.0,
            screen_x: 50.0,
            screen_y: 43.0,
        };
        adjust_coords_for_subdocument(
            &mut coords,
            Point { x: 95.0, y: 205.0 },
            Point { x: 20.0, y: 30.0 },
        );
        assert_eq!((coords.client_x, coords.client_y), (7.0, 10.0));
        assert_eq!((coords.page_x, coords.page_y), (27.0, 40.0));
        assert_eq!((coords.screen_x, coords.screen_y), (50.0, 43.0));
    }
}
