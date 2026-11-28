use imperifex_domain::i18n::{text, Language, Text};
use imperifex_domain::theme::{palette, Theme};
use imperifex_domain::view::Pointer;

use super::*;
use crate::testing::FixedWidth;

/// A window of 800 by 600 at 100 %; the fake font's rows are 20 high with
/// letters of 16, and a character is 16 wide.
fn model(view: EditorView) -> Model {
    Model {
        title: String::new(),
        tab: None,
        language: Language::Spanish,
        palette: palette(Theme::Dark),
        scale: 1.0,
        window: (800, 600),
        maximized: false,
        focused: true,
        full_screen: false,
        notice: Vec::new(),
        editor: Some(view),
    }
}

fn view(command: &str) -> EditorView {
    EditorView {
        prompt: vec!["PS>".into()],
        lines: command.split('\n').map(str::to_string).collect(),
        cursor: (0, command.chars().count()),
        letters: 16,
        ..EditorView::default()
    }
}

fn frame_of(model: &Model) -> crate::Frame {
    crate::frame(model, &Pointer::default(), &FixedWidth)
}

fn run<'a>(list: &'a DrawList, said: &str) -> &'a imperifex_domain::draw::TextRun {
    let found = list.texts.iter().find(|run| run.text == said);
    found.unwrap_or_else(|| panic!("{said:?} is not drawn"))
}

#[test]
fn the_console_keeps_its_size_and_the_output_ends_over_the_editor() {
    let with = frame_of(&model(view("ls")));
    let mut without = model(view("ls"));
    without.editor = None;
    let without = frame_of(&without);
    assert_eq!(with.terminal_area, without.terminal_area);
    assert_eq!(without.output_area, without.terminal_area);
    // From the status bar at 572: 1 + 12 + (24 + 2 rows of 20 + 8) + 8 + 16
    // + 12 is 121, and 12 more over it.
    let area = with.output_area;
    assert_eq!((area.y, area.y + area.height as i32), (56, 439));
}

#[test]
fn the_prompt_is_over_the_command_in_a_raised_box_with_the_accent_around() {
    let model = model(view("git"));
    let found = frame_of(&model).draw;
    let prompt = run(&found, "PS>");
    let command = run(&found, "git");
    assert_eq!((prompt.x, command.x), (28, 28));
    assert_eq!(command.y - prompt.y, 28, "a row and (S)");
    assert_eq!(prompt.color, model.palette.accent);
    assert_eq!(command.color, model.palette.text);
    assert_eq!((prompt.font, prompt.size), (UiFont::Mono, 16));
    let colours: Vec<_> = found.rounded.iter().map(|shape| shape.color).collect();
    let palette = model.palette;
    assert_eq!(colours[..2], [palette.accent, palette.raised]);
    assert_eq!(found.rounded[0].area.x, 16);
    assert_eq!(found.rounded[0].area.width, 768);
    assert_eq!((found.rounded[0].radius, found.rounded[1].radius), (8, 7));
}

#[test]
fn the_cursor_is_a_bar_after_what_is_before_it_and_the_suggestion_starts_there() {
    let mut typed = view("git");
    typed.suggestion = " push".into();
    let model = model(typed);
    let found = frame_of(&model).draw;
    let bar = found.rounded.last().expect("the cursor");
    let command = run(&found, "git");
    assert_eq!((bar.area.x, bar.area.y), (28 + 48, command.y));
    assert_eq!((bar.area.width, bar.area.height, bar.radius), (2, 20, 0));
    let grey = run(&found, " push");
    assert_eq!((grey.x, grey.y), (bar.area.x, command.y));
    assert_eq!(grey.color, model.palette.suggestion);
}

#[test]
fn without_the_keyboard_there_is_no_cursor_and_the_box_dims_its_line() {
    let mut unfocused = model(view("x"));
    unfocused.focused = false;
    let found = frame_of(&unfocused).draw;
    let colours: Vec<_> = found.rounded.iter().map(|shape| shape.color).collect();
    assert_eq!(colours, [unfocused.palette.line, unfocused.palette.raised]);
}

#[test]
fn a_line_more_makes_the_box_a_row_taller() {
    let one = frame_of(&model(view("a"))).output_area.height;
    let two = frame_of(&model(view("a\nb"))).output_area.height;
    assert_eq!(one - two, 20);
}

#[test]
fn under_the_box_the_keys_are_told_in_the_language_or_why_it_was_not_sent() {
    for language in [Language::Spanish, Language::English] {
        let mut model = model(view("a"));
        model.language = language;
        let found = frame_of(&model).draw;
        let key = run(&found, text(language, Text::KeyShiftEnter));
        let does = run(&found, text(language, Text::EditorNewLine));
        assert_eq!(key.color, model.palette.text);
        assert_eq!(does.color, model.palette.muted);
        assert_eq!(key.y, does.y);
        assert!(does.x > key.x);
    }
    let mut refused = view("a\nb");
    refused.refused = true;
    let model = model(refused);
    let found = frame_of(&model).draw;
    let why = run(&found, text(Language::Spanish, Text::EditorRefused));
    assert_eq!(why.color, model.palette.warning);
    let run_hint = text(Language::Spanish, Text::EditorRun);
    assert!(found.texts.iter().all(|run| run.text != run_hint));
}
