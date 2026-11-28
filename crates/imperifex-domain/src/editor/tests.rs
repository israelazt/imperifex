use super::*;

fn editor(history: &[&str]) -> Editor {
    Editor::new(history.iter().map(ToString::to_string).collect())
}

fn write(editor: &mut Editor, text: &str) {
    editor.apply(EditorKey::Insert(text.into()));
}

fn press(editor: &mut Editor, keys: &[EditorKey]) {
    for key in keys {
        editor.apply(key.clone());
    }
}

use EditorKey::{
    Backspace, Clear, Delete, Down, End, Home, Interrupt, Left, NewLine, Right, Run, Up,
};

#[test]
fn text_goes_where_the_cursor_is_and_is_removed_a_character_at_a_time() {
    let mut e = editor(&[]);
    write(&mut e, "año");
    press(&mut e, &[Left, Left]);
    write(&mut e, "€");
    assert_eq!((e.text(), e.cursor()), ("a€ño", (0, 2)));
    press(&mut e, &[Backspace, Delete]);
    assert_eq!((e.text(), e.cursor()), ("ao", (0, 1)));
    press(&mut e, &[Home]);
    assert_eq!(e.apply(Backspace), Outcome::Unchanged);
    press(&mut e, &[End]);
    assert_eq!(e.apply(Delete), Outcome::Unchanged);
}

#[test]
fn with_several_lines_the_arrows_move_between_them_and_home_and_end_stay() {
    let mut e = editor(&["old"]);
    write(&mut e, "first");
    press(&mut e, &[NewLine]);
    write(&mut e, "ab");
    assert_eq!(e.cursor(), (1, 2));
    press(&mut e, &[Up]);
    assert_eq!(e.cursor(), (0, 2));
    press(&mut e, &[End, Down]);
    assert_eq!(e.cursor(), (1, 2), "a shorter line takes its end");
    press(&mut e, &[Home]);
    assert_eq!(e.cursor(), (1, 0));
    assert_eq!(e.text(), "first\nab");
}

#[test]
fn up_goes_back_through_the_history_and_down_returns_to_what_was_written() {
    let mut e = editor(&["one", "two"]);
    write(&mut e, "dra");
    press(&mut e, &[Up]);
    assert_eq!(e.text(), "two");
    press(&mut e, &[Up, Up, Up]);
    assert_eq!(e.text(), "one", "it stays on the oldest");
    press(&mut e, &[Down]);
    assert_eq!(e.text(), "two");
    press(&mut e, &[Down, Down]);
    assert_eq!((e.text(), e.cursor()), ("dra", (0, 3)));
    let mut empty = editor(&[]);
    assert_eq!(empty.apply(Up), Outcome::Unchanged);
}

#[test]
fn a_command_looked_at_and_then_edited_is_what_is_written_now() {
    let mut e = editor(&["one", "two"]);
    press(&mut e, &[Up]);
    write(&mut e, "!");
    press(&mut e, &[Up, Down]);
    assert_eq!(e.text(), "two!");
}

#[test]
fn the_suggestion_is_the_rest_of_the_most_recent_command_that_starts_so() {
    let mut e = editor(&["git push origin old", "git status", "git push origin main"]);
    assert_eq!(e.suggestion(), "", "nothing is suggested to nothing");
    write(&mut e, "git pu");
    assert_eq!(e.suggestion(), "sh origin main");
    press(&mut e, &[Left]);
    assert_eq!(e.suggestion(), "", "only at the end");
    press(&mut e, &[Right]);
    assert_eq!(e.text(), "git pu");
    press(&mut e, &[Right]);
    assert_eq!((e.text(), e.suggestion()), ("git push origin main", ""));
    assert_eq!(e.apply(Right), Outcome::Unchanged);
}

#[test]
fn enter_asks_to_run_and_the_text_stays_until_it_was_sent() {
    let mut e = editor(&[]);
    write(&mut e, "ls");
    assert_eq!(e.apply(Run), Outcome::Run("ls".into()));
    assert_eq!(e.text(), "ls");
    e.sent(true);
    assert_eq!((e.text(), e.cursor()), ("", (0, 0)));
    press(&mut e, &[Up]);
    assert_eq!(e.text(), "ls");
    e.sent(false);
    press(&mut e, &[Up]);
    assert_eq!(e.text(), "ls", "what is not kept is not remembered");
}

#[test]
fn ctrl_c_and_escape_empty_it_and_ctrl_c_with_nothing_is_for_the_shell() {
    let mut e = editor(&[]);
    for key in [Interrupt, Clear] {
        write(&mut e, "x");
        assert_eq!(e.apply(key), Outcome::Changed);
        assert_eq!(e.text(), "");
    }
    assert_eq!(e.apply(Clear), Outcome::Unchanged);
    assert_eq!(e.apply(Interrupt), Outcome::Interrupt);
}

fn input(text: Option<&str>, key: Option<NamedKey>, shift: bool) -> KeyInput {
    KeyInput {
        text: text.map(str::to_string),
        key,
        shift,
    }
}

#[test]
fn each_key_asks_the_editor_for_its_thing() {
    let named = |key, shift| editor_key(&input(None, Some(key), shift));
    assert_eq!(named(NamedKey::Enter, false), Some(Run));
    assert_eq!(named(NamedKey::Enter, true), Some(NewLine));
    assert_eq!(
        named(NamedKey::Tab, false),
        Some(EditorKey::Insert("    ".into()))
    );
    assert_eq!(named(NamedKey::Escape, false), Some(Clear));
    let typed = |text| editor_key(&input(Some(text), None, false));
    assert_eq!(typed("@"), Some(EditorKey::Insert("@".into())));
    assert_eq!(typed("\u{3}"), Some(Interrupt));
    // Other control characters, such as Ctrl+L, are not text.
    assert_eq!(typed("\u{c}"), None);
    assert_eq!(editor_key(&KeyInput::default()), None);
}
