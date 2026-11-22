use blitz_traits::events::{BlitzImeEvent, DomEvent};

use crate::BaseDocument;

pub(crate) fn handle_ime_event<F: FnMut(DomEvent)>(
    doc: &mut BaseDocument,
    event: BlitzImeEvent,
    dispatch_event: F,
) {
    if let Some(node_id) = doc.focus_node_id {
        let node = &mut doc.nodes[node_id];
        let text_input_data = node
            .data
            .downcast_element_mut()
            .and_then(|el| el.text_input_data_mut());
        if let Some(input_data) = text_input_data {
            let generated_event = input_data.apply_ime_event(
                &mut doc.font_ctx.lock().unwrap(),
                &mut doc.layout_ctx,
                event,
            );

            if let Some(generated_event) = generated_event {
                doc.apply_generated_text_input_event(node_id, generated_event, dispatch_event);
            }

            #[cfg(feature = "tracing")]
            tracing::debug!(node_id = ?node_id, "Sent ime event");
        }
    }
}

pub(super) fn editing_operation(event: &BlitzImeEvent) -> Option<(&'static str, Option<String>)> {
    match event {
        BlitzImeEvent::Commit(text) => Some(("insertCompositionText", Some(text.clone()))),
        BlitzImeEvent::DeleteSurrounding {
            before_bytes,
            after_bytes,
        } if *before_bytes != 0 || *after_bytes != 0 => Some((
            if *before_bytes != 0 {
                "deleteContentBackward"
            } else {
                "deleteContentForward"
            },
            None,
        )),
        _ => None,
    }
}
