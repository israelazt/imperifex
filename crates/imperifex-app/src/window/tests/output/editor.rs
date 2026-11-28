//! The command editor inside the window. The marks a shell writes are read by
//! the real parser and the domain; here the terminal is told what it waits for.

use imperifex_domain::input::{KeyInput, NamedKey};
use imperifex_domain::shell_state::ShellState;

use super::*;
use crate::testing::{FakeClipboard, FakeHistory};

fn typed(text: &str) -> KeyInput {
    KeyInput {
        text: Some(text.into()),
        ..KeyInput::default()
    }
}

fn key(key: NamedKey, shift: bool) -> KeyInput {
    KeyInput {
        key: Some(key),
        shift,
        ..KeyInput::default()
    }
}

fn enter() -> KeyInput {
    key(NamedKey::Enter, false)
}

/// A window with `PowerShell` and the integration, and room for 40 by 10.
fn window(history: &FakeHistory, settings: Settings) -> (Core, FakeLauncher) {
    let appearance = FakeAppearance::dark(true);
    let mut core = Core::new(settings, Language::English, Box::new(appearance));
    core.keep_history(Box::new(history.clone()));
    core.resized((800, 600), false);
    core.fit(area(400, 200), area(400, 200), CELL, ms(0));
    let launcher = FakeLauncher::default();
    assert!(open(&mut core, &launcher));
    (core, launcher)
}

fn powershell() -> Settings {
    Settings {
        shell: Some("powershell".into()),
        ..Settings::default()
    }
}

/// The shell wrote its prompt and waits.
fn wait(core: &mut Core) {
    let tab = core.output.tab_mut().expect("a shell");
    tab.terminal_mut().shell = ShellState::Waiting(vec!["PS>".into()]);
}

fn waiting(history: &FakeHistory) -> (Core, FakeLauncher) {
    let (mut core, launcher) = window(history, powershell());
    wait(&mut core);
    (core, launcher)
}

fn sent(launcher: &FakeLauncher) -> String {
    String::from_utf8_lossy(&launcher.asked.borrow().written).into_owned()
}

#[test]
fn the_editor_is_shown_only_while_the_shell_waits_and_nothing_took_the_screen_or_the_mouse() {
    let (mut core, _) = window(&FakeHistory::default(), powershell());
    assert_eq!(core.model().editor, None, "it has said nothing yet");
    wait(&mut core);
    let shown = core.model().editor.expect("the editor");
    assert_eq!(shown.prompt, ["PS>"]);
    assert_eq!((shown.lines, shown.letters), (vec![String::new()], 16));
    let tab = core.output.tab_mut().expect("a shell");
    tab.terminal_mut().modes.mouse_reports = true;
    assert_eq!(core.model().editor, None);
    let tab = core.output.tab_mut().expect("a shell");
    tab.terminal_mut().modes.mouse_reports = false;
    tab.terminal_mut().enter_alternate_screen();
    assert_eq!(core.model().editor, None);
}

#[test]
fn what_is_typed_stays_in_the_editor_until_enter_sends_it_whole() {
    let history = FakeHistory::default();
    let (mut core, launcher) = waiting(&history);
    for text in ["e", "c", "h", "o"] {
        assert_eq!(core.typed(&typed(text)), [WindowCommand::Redraw]);
    }
    assert_eq!(sent(&launcher), "", "nothing reached the shell");
    assert_eq!(core.model().editor.expect("shown").lines, ["echo"]);
    assert_eq!(core.typed(&enter()), [WindowCommand::Redraw]);
    assert_eq!(sent(&launcher), "echo\r");
    // The shell runs it: from now on keys are its own.
    assert_eq!(core.model().editor, None);
    core.typed(&typed("y"));
    assert_eq!(sent(&launcher), "echo\ry");
    assert_eq!(history.added(), [("powershell".into(), "echo\n".into())]);
}

#[test]
fn an_empty_enter_gets_another_prompt_and_what_starts_with_a_space_is_not_kept() {
    let history = FakeHistory::default();
    let (mut core, launcher) = waiting(&history);
    core.typed(&enter());
    assert!(core.model().editor.is_some(), "it is still waiting");
    core.typed(&typed(" secret"));
    core.typed(&enter());
    assert_eq!(sent(&launcher), "\r secret\r");
    assert!(history.added().is_empty());
}

#[test]
fn the_history_of_the_profile_suggests_and_comes_back_with_up() {
    let history = FakeHistory::holding("git status\ngit push\n");
    let (mut core, _) = waiting(&history);
    core.typed(&typed("git s"));
    assert_eq!(core.model().editor.expect("shown").suggestion, "tatus");
    core.typed(&key(NamedKey::Escape, false));
    core.typed(&key(NamedKey::Up, false));
    assert_eq!(core.model().editor.expect("shown").lines, ["git push"]);
}

#[test]
fn several_lines_reach_powershell_with_shift_enter_between_them() {
    let (mut core, launcher) = waiting(&FakeHistory::default());
    core.typed(&typed("a"));
    core.typed(&key(NamedKey::Enter, true));
    core.typed(&typed("b"));
    assert_eq!(core.model().editor.expect("shown").lines, ["a", "b"]);
    core.typed(&enter());
    let found = sent(&launcher);
    assert!(found.starts_with("a\x1b[13;28;13;1;16;1_"), "{found:?}");
    assert!(found.ends_with("b\r"));
}

#[test]
fn ctrl_c_with_nothing_written_interrupts_the_shell() {
    let (mut core, launcher) = waiting(&FakeHistory::default());
    core.typed(&typed("x"));
    core.typed(&typed("\u{3}"));
    assert_eq!(sent(&launcher), "");
    core.typed(&typed("\u{3}"));
    assert_eq!(sent(&launcher), "\x03");
}

#[test]
fn turned_off_the_keys_go_to_the_shell_and_turned_on_the_text_is_still_there() {
    let (mut core, launcher) = waiting(&FakeHistory::default());
    core.typed(&typed("half"));
    assert_eq!(core.toggle_editor(), [WindowCommand::Redraw]);
    assert_eq!(core.model().editor, None);
    core.typed(&typed("direct"));
    assert_eq!(sent(&launcher), "direct");
    core.toggle_editor();
    assert_eq!(core.model().editor.expect("shown").lines, ["half"]);
}

#[test]
fn without_the_integration_or_with_the_editor_off_in_the_settings_there_is_none() {
    let without = Settings {
        shell_integration: false,
        ..powershell()
    };
    let off = Settings {
        command_editor: false,
        ..powershell()
    };
    for settings in [without, off] {
        let (mut core, _) = window(&FakeHistory::default(), settings);
        wait(&mut core);
        assert_eq!(core.model().editor, None);
    }
}

#[test]
fn with_the_editor_the_output_ends_where_its_room_does_without_the_prompts_row() {
    let (mut core, _) = window(&FakeHistory::default(), powershell());
    let tab = core.output.tab_mut().expect("a shell");
    // Two lines of output and the prompt under them, on the third row.
    let screen = tab.terminal_mut().screen_mut();
    for _ in 0..2 {
        screen.print('o');
        screen.carriage_return();
        screen.line_feed();
    }
    screen.print('>');
    wait(&mut core);
    // The editor leaves room for 5 rows of 20, from 56 to 156.
    core.fit(area(400, 200), area(400, 100), CELL, ms(0));
    let damage = core.take_damage();
    let picture = core.picture(&damage).expect("a picture");
    assert_eq!(picture.first, 0);
    let expected = Rect {
        y: 116,
        height: 40,
        ..area(400, 100)
    };
    assert_eq!(picture.area, expected);
    // While a command runs, the cursor's row is drawn too.
    let tab = core.output.tab_mut().expect("a shell");
    tab.terminal_mut().shell = ShellState::Running;
    core.fit(area(400, 200), area(400, 200), CELL, ms(0));
    let picture = core.picture(&damage).expect("a picture");
    assert_eq!((picture.area.y, picture.area.height), (196, 60));
}

#[test]
fn a_paste_is_written_in_the_editor_and_runs_nothing() {
    let (mut core, launcher) = waiting(&FakeHistory::default());
    core.use_clipboard(Box::new(FakeClipboard::holding("a\r\nb\r\n")));
    assert_eq!(core.paste(), [WindowCommand::Redraw]);
    assert_eq!(core.model().editor.expect("shown").lines, ["a", "b"]);
    assert_eq!(sent(&launcher), "");
}

#[test]
fn several_lines_into_a_program_warn_and_go_only_if_pasted_again() {
    let (mut core, launcher) = window(&FakeHistory::default(), powershell());
    core.use_clipboard(Box::new(FakeClipboard::holding("a\nb")));
    assert_eq!(core.paste(), [WindowCommand::Redraw]);
    assert_eq!(sent(&launcher), "");
    let warning = core.model().warning.expect("a warning");
    assert!(warning.contains('2'), "{warning}");
    // Anything else typed is a no: it is asked again.
    core.typed(&typed("x"));
    assert_eq!(core.model().warning, None);
    core.paste();
    assert_eq!(sent(&launcher), "x");
    core.paste();
    assert_eq!(sent(&launcher), "xa\rb");
    assert_eq!(core.model().warning, None);
}

#[test]
fn a_click_in_the_editor_moves_its_cursor() {
    let (mut core, _) = waiting(&FakeHistory::default());
    core.typed(&typed("hola"));
    let place = Action::PlaceCursor { line: 0, column: 1 };
    assert_eq!(core.apply(place), [WindowCommand::Redraw]);
    assert_eq!(core.model().editor.expect("shown").cursor, (0, 1));
}
