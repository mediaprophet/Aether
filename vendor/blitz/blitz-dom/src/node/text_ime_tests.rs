// Copyright (c) 2026 Timothy Charles Holborn. Repository licensing applies.
use super::*;

struct Editor {
    input: TextInputData,
    fonts: FontContext,
    layout: LayoutContext<TextBrush>,
}

impl Editor {
    fn new(text: &str, anchor: usize, focus: usize) -> Self {
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
            .select_byte_range(anchor, focus);
        editor
    }

    fn ime(&mut self, event: BlitzImeEvent) -> Option<GeneratedTextInputEvent> {
        self.input
            .apply_ime_event(&mut self.fonts, &mut self.layout, event)
    }

    fn delete(
        &mut self,
        before_bytes: usize,
        after_bytes: usize,
    ) -> Option<GeneratedTextInputEvent> {
        self.ime(BlitzImeEvent::DeleteSurrounding {
            before_bytes,
            after_bytes,
        })
    }
}

#[test]
fn surrounding_delete_preserves_unicode_selection_and_direction() {
    for (anchor, focus, expected_anchor, expected_focus) in [(2, 4, 0, 2), (4, 2, 2, 0)] {
        let mut editor = Editor::new("éab🙂", anchor, focus);
        assert!(matches!(
            editor.delete(2, 4),
            Some(GeneratedTextInputEvent::Input)
        ));
        assert_eq!(editor.input.editor.text(), "ab");
        assert_eq!(editor.input.editor.selected_text(), Some("ab"));
        assert_eq!(
            editor.input.editor.raw_selection().anchor().index(),
            expected_anchor
        );
        assert_eq!(
            editor.input.editor.raw_selection().focus().index(),
            expected_focus
        );
    }
}

#[test]
fn surrounding_delete_rejects_split_utf8_on_either_side_atomically() {
    for (before, after) in [(1, 4), (2, 1), (0, 0)] {
        let mut editor = Editor::new("éab🙂", 4, 2);
        assert!(editor.delete(before, after).is_none());
        assert_eq!(editor.input.editor.raw_text(), "éab🙂");
        assert_eq!(editor.input.editor.raw_selection().anchor().index(), 4);
        assert_eq!(editor.input.editor.raw_selection().focus().index(), 2);
    }
}

#[test]
fn surrounding_delete_clamps_large_lengths_without_deleting_selection() {
    let mut editor = Editor::new("éab🙂", 2, 4);
    assert!(matches!(
        editor.delete(usize::MAX, usize::MAX),
        Some(GeneratedTextInputEvent::Input)
    ));
    assert_eq!(editor.input.editor.text(), "ab");
    assert!(editor.delete(usize::MAX, usize::MAX).is_none());
}

#[test]
fn surrounding_delete_preserves_preedit_cursor_and_later_commit() {
    for cursor in [Some((3, 6)), None] {
        let mut editor = Editor::new("é🙂", 2, 2);
        editor.ime(BlitzImeEvent::Preedit("日本".into(), cursor));
        assert!(matches!(
            editor.delete(2, 4),
            Some(GeneratedTextInputEvent::Input)
        ));
        assert_eq!(editor.input.editor.raw_text(), "日本");
        assert_eq!(editor.input.editor.text(), "");
        assert_eq!(*editor.input.editor.raw_compose(), Some(0..6));
        assert_eq!(editor.input.editor.is_cursor_visible(), cursor.is_some());
        let (anchor, focus) = cursor.unwrap_or((0, 0));
        assert_eq!(editor.input.editor.raw_selection().anchor().index(), anchor);
        assert_eq!(editor.input.editor.raw_selection().focus().index(), focus);
        editor.ime(BlitzImeEvent::Preedit(String::new(), None));
        assert!(matches!(
            editor.ime(BlitzImeEvent::Commit("語".into())),
            Some(GeneratedTextInputEvent::Input)
        ));
        assert_eq!(editor.input.editor.text(), "語");
        assert!(!editor.input.editor.is_composing());
    }
}

#[test]
fn surrounding_delete_preserves_preedit_across_a_combining_cluster_boundary() {
    let mut editor = Editor::new("aX", 1, 1);
    editor.ime(BlitzImeEvent::Preedit("\u{301}".into(), None));
    assert_eq!(editor.input.editor.raw_text(), "a\u{301}X");
    assert_eq!(*editor.input.editor.raw_compose(), Some(1..3));
    assert!(matches!(
        editor.delete(1, 1),
        Some(GeneratedTextInputEvent::Input)
    ));
    assert_eq!(editor.input.editor.raw_text(), "\u{301}");
    assert_eq!(*editor.input.editor.raw_compose(), Some(0..2));
    assert_eq!(editor.input.editor.text(), "");
    assert!(!editor.input.editor.is_cursor_visible());
}

#[test]
fn rejected_surrounding_delete_preserves_active_composition() {
    let mut editor = Editor::new("é🙂", 2, 2);
    editor.ime(BlitzImeEvent::Preedit("日本".into(), Some((3, 6))));
    assert!(editor.delete(2, 1).is_none());
    assert_eq!(editor.input.editor.raw_text(), "é日本🙂");
    assert_eq!(*editor.input.editor.raw_compose(), Some(2..8));
    assert_eq!(editor.input.editor.raw_selection().anchor().index(), 5);
    assert_eq!(editor.input.editor.raw_selection().focus().index(), 8);
}
