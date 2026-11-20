// Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
use super::*;
use blitz_traits::{events::KeyState, shell::ClipboardError};
use keyboard_types::{Code, Location};
use std::sync::Mutex;

struct Clipboard {
    text: Mutex<String>,
    available: bool,
}

impl Clipboard {
    fn new(text: &str, available: bool) -> Self {
        Self {
            text: Mutex::new(text.into()),
            available,
        }
    }
}

impl ShellProvider for Clipboard {
    fn get_clipboard_text(&self) -> Result<String, ClipboardError> {
        if !self.available {
            return Err(ClipboardError);
        }
        Ok(self.text.lock().unwrap().clone())
    }

    fn set_clipboard_text(&self, text: String) -> Result<(), ClipboardError> {
        if !self.available {
            return Err(ClipboardError);
        }
        *self.text.lock().unwrap() = text;
        Ok(())
    }
}

struct Editor {
    input: TextInputData,
    fonts: FontContext,
    layout: LayoutContext<TextBrush>,
}

impl Editor {
    fn selected(text: &str) -> Self {
        let mut editor = Self {
            input: TextInputData::new(false),
            fonts: FontContext::default(),
            layout: LayoutContext::new(),
        };
        editor
            .input
            .set_text(&mut editor.fonts, &mut editor.layout, text);
        editor
            .input
            .editor
            .driver(&mut editor.fonts, &mut editor.layout)
            .select_all();
        editor
    }

    fn shortcut(&mut self, key: &str, clipboard: &Clipboard) -> Option<GeneratedTextInputEvent> {
        self.input.apply_keypress_event(
            &mut self.fonts,
            &mut self.layout,
            clipboard,
            BlitzKeyEvent {
                key: Key::Character(key.into()),
                code: Code::Unidentified,
                modifiers: ACTION_MOD,
                location: Location::Standard,
                is_auto_repeating: false,
                is_composing: false,
                state: KeyState::Pressed,
                text: None,
            },
        )
    }
}

#[test]
fn copy_preserves_selection_and_never_emits_input() {
    for available in [false, true] {
        let clipboard = Clipboard::new("previous", available);
        let mut editor = Editor::selected("hello 日本");
        assert!(editor.shortcut("C", &clipboard).is_none());
        assert_eq!(editor.input.editor.text(), "hello 日本");
        assert_eq!(editor.input.editor.selected_text(), Some("hello 日本"));
        assert_eq!(
            *clipboard.text.lock().unwrap(),
            if available {
                "hello 日本"
            } else {
                "previous"
            }
        );
    }
}

#[test]
fn unavailable_clipboard_cannot_destroy_cut_or_paste_selection() {
    for key in ["x", "v", "X", "V"] {
        let clipboard = Clipboard::new("unreadable", false);
        let mut editor = Editor::selected("keep é日本");
        assert!(editor.shortcut(key, &clipboard).is_none());
        assert_eq!(editor.input.editor.text(), "keep é日本");
        assert_eq!(editor.input.editor.selected_text(), Some("keep é日本"));
        assert_eq!(*clipboard.text.lock().unwrap(), "unreadable");
    }
}

#[test]
fn successful_cut_and_paste_emit_one_edit_result_each() {
    let clipboard = Clipboard::new("previous", true);
    let mut editor = Editor::selected("cut 日本");
    assert!(matches!(
        editor.shortcut("X", &clipboard),
        Some(GeneratedTextInputEvent::Input)
    ));
    assert_eq!(editor.input.editor.text(), "");
    assert_eq!(*clipboard.text.lock().unwrap(), "cut 日本");
    assert!(editor.shortcut("x", &clipboard).is_none());
    assert!(matches!(
        editor.shortcut("V", &clipboard),
        Some(GeneratedTextInputEvent::Input)
    ));
    assert_eq!(editor.input.editor.text(), "cut 日本");
}

#[test]
fn empty_clipboard_does_not_delete_selection() {
    let clipboard = Clipboard::new("", true);
    let mut editor = Editor::selected("keep selection");
    assert!(editor.shortcut("v", &clipboard).is_none());
    assert_eq!(editor.input.editor.text(), "keep selection");
    assert_eq!(editor.input.editor.selected_text(), Some("keep selection"));
}
