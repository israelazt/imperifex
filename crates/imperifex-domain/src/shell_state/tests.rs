use super::*;
use crate::interpret::apply;
use crate::sequence::Sequence;

fn mark(terminal: &mut Terminal, letter: &str) {
    let parts: [&[u8]; 2] = [b"133", letter.as_bytes()];
    apply(terminal, Sequence::Osc(&parts));
}

fn write(terminal: &mut Terminal, text: &str) {
    for character in text.chars() {
        let sequence = match character {
            '\r' | '\n' => Sequence::Control(character as u8),
            other => Sequence::Print(other),
        };
        apply(terminal, sequence);
    }
}

fn waiting(lines: &[&str]) -> ShellState {
    ShellState::Waiting(lines.iter().map(ToString::to_string).collect())
}

/// A shell that wrote `prompt` between its two marks.
fn prompted(prompt: &str) -> Terminal {
    let mut terminal = Terminal::new(20, 4);
    write(&mut terminal, "out\r\n");
    mark(&mut terminal, "A");
    write(&mut terminal, prompt);
    mark(&mut terminal, "B");
    terminal
}

#[test]
fn the_prompt_is_what_was_written_between_its_marks() {
    assert_eq!(prompted("PS C:\\> ").shell, waiting(&["PS C:\\>"]));
    let two = prompted("host ~/src\r\n$ ");
    assert_eq!(two.shell, waiting(&["host ~/src", "$"]));
}

#[test]
fn a_prompt_that_scrolls_the_screen_while_it_is_written_is_whole() {
    let mut terminal = Terminal::new(20, 3);
    write(&mut terminal, "1\r\n2\r\n");
    mark(&mut terminal, "A");
    write(&mut terminal, "top\r\nmiddle\r\n> ");
    mark(&mut terminal, "B");
    assert_eq!(terminal.shell, waiting(&["top", "middle", ">"]));
}

#[test]
fn running_a_command_ends_the_wait_and_the_next_prompt_starts_another() {
    let mut terminal = prompted("> ");
    mark(&mut terminal, "C");
    assert_eq!(terminal.shell, ShellState::Running);
    // How it ended changes nothing yet.
    mark(&mut terminal, "D");
    assert_eq!(terminal.shell, ShellState::Running);
    mark(&mut terminal, "A");
    write(&mut terminal, "$ ");
    mark(&mut terminal, "B");
    assert_eq!(terminal.shell, waiting(&["$"]));
}

#[test]
fn without_marks_nothing_is_known_and_a_b_alone_or_other_codes_say_nothing() {
    let mut terminal = Terminal::new(20, 4);
    write(&mut terminal, "> ");
    mark(&mut terminal, "B");
    let parts: [&[u8]; 2] = [b"7", b"A"];
    apply(&mut terminal, Sequence::Osc(&parts));
    assert_eq!(terminal.shell, ShellState::Unknown);
}

#[test]
fn the_marks_of_a_program_on_the_whole_screen_are_not_the_shells_and_a_reset_forgets() {
    let mut terminal = prompted("> ");
    terminal.enter_alternate_screen();
    mark(&mut terminal, "C");
    assert_eq!(terminal.shell, waiting(&[">"]));
    terminal.reset();
    assert_eq!(terminal.shell, ShellState::Unknown);
}
