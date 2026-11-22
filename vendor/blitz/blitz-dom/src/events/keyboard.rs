use crate::events::focus::generate_focus_events;
use crate::{BaseDocument, node::GeneratedTextInputEvent, util::ACTION_MOD};
use blitz_traits::node_id::NodeId;
use blitz_traits::{
    SmolStr,
    events::{
        BlitzBeforeInputEvent, BlitzCancelEvent, BlitzClipboardEvent, BlitzFormActionEvent,
        BlitzInputEvent, BlitzKeyEvent, BlitzSelectionEvent, DomEvent, DomEventData,
    },
};
use keyboard_types::{Key, Modifiers, NamedKey};
use markup5ever::local_name;

pub(super) enum KeyboardOrTextInputEvent {
    KeyPress(BlitzKeyEvent),
    AppleStandardKeyBinding(SmolStr),
}

pub(crate) fn handle_key_or_input_event<F: FnMut(DomEvent)>(
    doc: &mut BaseDocument,
    target: NodeId,
    event: KeyboardOrTextInputEvent,
    mut dispatch_event: F,
) {
    if let KeyboardOrTextInputEvent::KeyPress(event) = &event {
        if event.key == Key::Named(NamedKey::Tab) {
            let backwards = event.modifiers.contains(Modifiers::SHIFT);
            let old_focus = doc.focus_node_id;
            generate_focus_events(
                doc,
                &mut |doc| {
                    if backwards {
                        doc.focus_prev_node();
                    } else {
                        doc.focus_next_node();
                    }
                },
                &mut dispatch_event,
            );
            if doc.focus_node_id != old_focus {
                doc.shell_provider.request_redraw();
            }
            return;
        }
    }

    if let Some(node_id) = doc.focus_node_id {
        if target != node_id {
            return;
        }

        if !doc.nodes.contains_key(node_id) {
            return;
        }
        let node = &mut doc.nodes[node_id];
        let Some(element_data) = node.element_data_mut() else {
            return;
        };

        if let Some(input_data) = element_data.text_input_data_mut() {
            let generated_event = match event {
                KeyboardOrTextInputEvent::KeyPress(blitz_key_event) => input_data
                    .apply_keypress_event(
                        &mut doc.font_ctx.lock().unwrap(),
                        &mut doc.layout_ctx,
                        &*doc.shell_provider,
                        blitz_key_event,
                    ),
                KeyboardOrTextInputEvent::AppleStandardKeyBinding(command) => input_data
                    .apply_apple_standard_keybinding(
                        &mut doc.font_ctx.lock().unwrap(),
                        &mut doc.layout_ctx,
                        &*doc.shell_provider,
                        &command,
                    ),
            };

            if let Some(generated_event) = generated_event {
                doc.apply_generated_text_input_event(node_id, generated_event, dispatch_event);
            }
        }
    }
}

impl BaseDocument {
    pub(crate) fn apply_generated_text_input_event<F: FnMut(DomEvent)>(
        &mut self,
        node_id: NodeId,
        event: GeneratedTextInputEvent,
        mut dispatch_event: F,
    ) {
        let node = &mut self.nodes[node_id];
        let element_data = node
            .element_data_mut()
            .expect("apply_generated_text_input_event called on a node that is not an element");
        let input_data = element_data
            .text_input_data_mut()
            .expect("apply_generated_text_input_event called on a node that is not a text input");

        match event {
            GeneratedTextInputEvent::Input => {
                let value = input_data.editor.raw_text().to_string();
                dispatch_event(DomEvent::new(
                    node_id,
                    DomEventData::Input(BlitzInputEvent { value }),
                ));
                self.shell_provider.request_redraw();
            }
            GeneratedTextInputEvent::Select => {
                let range = input_data.editor.raw_selection().text_range();
                let text = input_data.editor.raw_text();
                let selection = BlitzSelectionEvent {
                    selected_text: input_data
                        .editor
                        .selected_text()
                        .unwrap_or_default()
                        .to_owned(),
                    start_offset: text[..range.start].encode_utf16().count(),
                    end_offset: text[..range.end].encode_utf16().count(),
                };
                dispatch_event(DomEvent::new(
                    node_id,
                    DomEventData::SelectionChange(selection.clone()),
                ));
                if range.start != range.end {
                    dispatch_event(DomEvent::new(node_id, DomEventData::Select(selection)));
                }
                self.shell_provider.request_redraw();
            }
            GeneratedTextInputEvent::PreEditChange => {
                self.shell_provider.request_redraw();
            }
            GeneratedTextInputEvent::Submit => {
                let value = input_data.editor.raw_text().to_string();
                if let Some(form_id) = implicit_form_target(self, node_id) {
                    dispatch_event(DomEvent::new(
                        form_id,
                        DomEventData::Submit(BlitzFormActionEvent { value }),
                    ));
                }
            }
        }
    }
}

/// https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#field-that-blocks-implicit-submission
fn implicit_form_target(doc: &BaseDocument, text_target: NodeId) -> Option<NodeId> {
    let form_owner_id = doc.controls_to_form.get(&text_target)?;
    if doc
        .controls_to_form
        .iter()
        .filter(|(_control_id, form_id)| *form_id == form_owner_id)
        .filter_map(|(control_id, _)| doc.nodes[*control_id].element_data())
        .filter(|element_data| {
            element_data.attr(local_name!("type")).is_some_and(|t| {
                matches!(
                    t,
                    "text"
                        | "search"
                        | "email"
                        | "url"
                        | "tel"
                        | "password"
                        | "date"
                        | "month"
                        | "week"
                        | "time"
                        | "datetime-local"
                        | "number"
                )
            })
        })
        .count()
        > 1
    {
        return None;
    }

    Some(*form_owner_id)
}

/// Build the synchronous events that precede an editor default action.
pub(super) fn editing_events(doc: &BaseDocument, event: &DomEvent) -> Vec<DomEvent> {
    let target = event.target;
    let input = doc
        .focus_node_id
        .filter(|id| *id == target)
        .and_then(|id| doc.get_node(id))
        .and_then(|node| node.element_data())
        .and_then(|element| element.text_input_data());

    let mut events = Vec::new();
    if let DomEventData::KeyDown(key) = &event.data {
        let printable = match &key.key {
            Key::Character(text) => !text.is_empty() && text.chars().all(|ch| !ch.is_control()),
            Key::Named(NamedKey::Enter) => true,
            _ => false,
        };
        if key.state.is_pressed()
            && !key.is_composing
            && printable
            && !key
                .modifiers
                .intersects(Modifiers::CONTROL | Modifiers::META)
        {
            events.push(DomEvent::new(target, DomEventData::KeyPress(key.clone())));
        }
    }

    let clipboard: Option<fn(BlitzClipboardEvent) -> DomEventData> = match &event.data {
        DomEventData::KeyDown(key)
            if key.state.is_pressed() && key.modifiers.contains(ACTION_MOD) =>
        {
            match &key.key {
                Key::Character(key) if key.eq_ignore_ascii_case("c") => Some(DomEventData::Copy),
                Key::Character(key) if key.eq_ignore_ascii_case("x") => Some(DomEventData::Cut),
                Key::Character(key) if key.eq_ignore_ascii_case("v") => Some(DomEventData::Paste),
                _ => None,
            }
        }
        DomEventData::AppleStandardKeybinding(command) if command == "yank:" => {
            Some(DomEventData::Cut)
        }
        _ => None,
    };
    if let Some(clipboard) = clipboard {
        let action = clipboard(BlitzClipboardEvent::default()).kind();
        let text = match (action, input) {
            (blitz_traits::events::DomEventKind::Paste, Some(_)) => {
                doc.shell_provider.get_clipboard_text().ok()
            }
            (_, Some(input)) => input.editor.selected_text().map(str::to_owned),
            (blitz_traits::events::DomEventKind::Copy, None) => doc.get_selected_text(),
            _ => return events,
        }
        .unwrap_or_default();
        let edit = match (action, input) {
            (blitz_traits::events::DomEventKind::Cut, Some(input)) if !text.is_empty() => {
                Some(before_input(input, "deleteByCut", None, false))
            }
            (blitz_traits::events::DomEventKind::Paste, Some(input)) if !text.is_empty() => Some(
                before_input(input, "insertFromPaste", Some(text.clone()), false),
            ),
            _ => None,
        };
        events.push(DomEvent::new(
            target,
            clipboard(BlitzClipboardEvent { text }),
        ));
        if let Some(edit) = edit {
            events.push(DomEvent::new(target, edit));
        }
        return events;
    }

    if let Some(input) = input {
        let operation = match &event.data {
            DomEventData::KeyDown(key) if key.state.is_pressed() => {
                let by_word = key.modifiers.contains(ACTION_MOD);
                match &key.key {
                    Key::Named(NamedKey::Delete) => Some((
                        if by_word {
                            "deleteWordForward"
                        } else {
                            "deleteContentForward"
                        },
                        None,
                    )),
                    #[cfg(not(target_os = "macos"))]
                    Key::Named(NamedKey::Backspace) => Some((
                        if by_word {
                            "deleteWordBackward"
                        } else {
                            "deleteContentBackward"
                        },
                        None,
                    )),
                    Key::Named(NamedKey::Enter) => input
                        .is_multiline
                        .then(|| ("insertLineBreak", Some("\n".into()))),
                    Key::Character(text) if text == "\n" => input
                        .is_multiline
                        .then(|| ("insertLineBreak", Some("\n".into()))),
                    Key::Character(text)
                        if !key
                            .modifiers
                            .intersects(Modifiers::CONTROL | Modifiers::META) =>
                    {
                        Some(("insertText", Some(text.to_string())))
                    }
                    _ => None,
                }
                .map(|(kind, data)| (kind, data, key.is_composing))
            }
            DomEventData::AppleStandardKeybinding(command) => match command.as_str() {
                "insertDoubleQuoteIgnoringSubstitution:" => Some(("insertText", Some("\"".into()))),
                "insertSingleQuoteIgnoringSubstitution:" => Some(("insertText", Some("'".into()))),
                "insertNewline:" if input.is_multiline => {
                    Some(("insertLineBreak", Some("\n".into())))
                }
                "insertLineBreak:"
                | "insertNewlineIgnoringFieldEditor:"
                | "insertParagraphSeparator:" => Some(("insertLineBreak", Some("\n".into()))),
                "deleteWordBackward:" => Some(("deleteWordBackward", None)),
                "deleteWordForward:" => Some(("deleteWordForward", None)),
                "deleteBackward:"
                | "deleteBackwardByDecomposingPreviousCharacter:"
                | "deleteToBeginningOfLine:"
                | "deleteToBeginningOfParagraph:" => Some(("deleteContentBackward", None)),
                "deleteForward:" | "deleteToEndOfLine:" | "deleteToEndOfParagraph:" => {
                    Some(("deleteContentForward", None))
                }
                _ => None,
            }
            .map(|(kind, data)| (kind, data, false)),
            DomEventData::Ime(event) => super::ime::editing_operation(event)
                .map(|(kind, text)| (kind, text, input.editor.raw_compose().is_some())),
            _ => None,
        };
        if let Some((kind, data, is_composing)) = operation {
            events.push(DomEvent::new(
                target,
                before_input(input, kind, data, is_composing),
            ));
        }
    }
    events
}

fn before_input(
    input: &crate::node::TextInputData,
    kind: &str,
    data: Option<String>,
    is_composing: bool,
) -> DomEventData {
    DomEventData::BeforeInput(BlitzBeforeInputEvent {
        input_type: kind.into(),
        data,
        is_composing,
        value: input.editor.raw_text().to_string(),
    })
}

pub(super) fn apply_clipboard(
    doc: &mut BaseDocument,
    target: NodeId,
    event: DomEventData,
    dispatch: impl FnMut(DomEvent),
) {
    let (text, cut) = match event {
        DomEventData::Copy(event) => {
            if !event.text.is_empty() {
                let _ = doc.shell_provider.set_clipboard_text(event.text);
            }
            return;
        }
        DomEventData::Cut(event) => (event.text, true),
        DomEventData::Paste(event) => (event.text, false),
        _ => return,
    };
    if text.is_empty() || doc.focus_node_id != Some(target) || !doc.nodes.contains_key(target) {
        return;
    }
    let Some(input) = doc.nodes[target]
        .element_data_mut()
        .and_then(|el| el.text_input_data_mut())
    else {
        return;
    };
    let mut font = doc.font_ctx.lock().unwrap();
    let mut editor = input.editor.driver(&mut font, &mut doc.layout_ctx);
    if cut {
        if editor.editor.selected_text() != Some(text.as_str())
            || doc.shell_provider.set_clipboard_text(text).is_err()
        {
            return;
        }
        editor.delete_selection();
    } else {
        editor.insert_or_replace_selection(&text);
    }
    drop(font);
    doc.apply_generated_text_input_event(target, GeneratedTextInputEvent::Input, dispatch);
    doc.clamp_text_input_scroll(target);
}

pub(super) fn dialog_cancel_event(doc: &BaseDocument, target: NodeId) -> Option<DomEvent> {
    doc.node_chain(target)
        .into_iter()
        .find(|id| {
            doc.get_node(*id).is_some_and(|node| {
                node.data.is_element_with_tag_name(&local_name!("dialog"))
                    && node.data.has_attr(local_name!("open"))
            })
        })
        .map(|id| DomEvent::new(id, DomEventData::Cancel(BlitzCancelEvent)))
}
