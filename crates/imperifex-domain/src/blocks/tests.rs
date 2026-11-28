use super::*;
use crate::interpret::apply;
use crate::sequence::Sequence;

fn mark(terminal: &mut Terminal, parts: &[&str]) {
    let mut all: Vec<&[u8]> = vec![b"133"];
    all.extend(parts.iter().map(|part| part.as_bytes()));
    apply(terminal, Sequence::Osc(&all));
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

/// A shell that shows a prompt and runs `command`, which writes `out` and ends
/// with `code` after 1500 ms.
fn run(terminal: &mut Terminal, command: &str, out: &str, code: &str) {
    mark(terminal, &["A"]);
    write(terminal, "> ");
    mark(terminal, &["B"]);
    write(terminal, command);
    write(terminal, "\r\n");
    mark(terminal, &["C"]);
    write(terminal, out);
    mark(terminal, &["D", code, "1500"]);
}

/// Two commands run and the next prompt, in a screen of 20 by 10.
fn two() -> Terminal {
    let mut terminal = Terminal::new(20, 10);
    run(&mut terminal, "ls", "a\r\nb\r\n", "0");
    run(&mut terminal, "no", "bad\r\n", "7");
    mark(&mut terminal, &["A"]);
    write(&mut terminal, "> ");
    mark(&mut terminal, &["B"]);
    terminal
}

fn spans(blocks: &[SeenBlock]) -> Vec<(u32, usize, usize, bool)> {
    blocks
        .iter()
        .map(|block| (block.id, block.from, block.to, block.starts_here))
        .collect()
}

#[test]
fn each_command_is_a_block_from_its_prompt_to_the_next_with_how_it_ended() {
    let found = seen(&two(), 0, 10);
    assert_eq!(
        spans(&found),
        [(1, 0, 3, true), (2, 3, 5, true), (3, 5, 10, true)]
    );
    let ran = |code| {
        Some(Ran {
            code,
            millis: Some(1500),
        })
    };
    let ends: Vec<_> = found.iter().map(|block| block.ran).collect();
    assert_eq!(ends, [ran(0), ran(7), None]);
    // The command is on the first row, 4 columns written: "> ls".
    let commands: Vec<_> = found.iter().map(|block| block.command).collect();
    assert_eq!(commands, [Some((0, 4)), Some((3, 4)), None]);
}

#[test]
fn what_a_command_wrote_can_be_had_without_its_prompt_or_the_next() {
    let terminal = two();
    assert_eq!(output(&terminal, 1).as_deref(), Some("a\nb"));
    assert_eq!(output(&terminal, 2).as_deref(), Some("bad"));
    assert_eq!(output(&terminal, 3), None, "nothing ran in it");
}

#[test]
fn blocks_travel_to_the_history_and_one_that_starts_above_the_view_is_found() {
    let mut terminal = Terminal::new(20, 3);
    run(&mut terminal, "count", "1\r\n2\r\n3\r\n4\r\n5\r\n", "0");
    let found = seen(&terminal, -2, 3);
    assert_eq!(spans(&found), [(1, 0, 3, false)]);
    assert_eq!(found[0].command, None, "its command is above");
    let all = output(&terminal, 1);
    assert_eq!(all.as_deref(), Some("1\n2\n3\n4\n5"));
    let history = isize::try_from(terminal.screen().history_len()).unwrap();
    let from_the_top = seen(&terminal, -history, 3);
    assert_eq!(from_the_top[0].command, Some((0, 7)));
}

#[test]
fn a_change_of_width_keeps_the_blocks_and_joins_the_lines_of_their_output() {
    let mut terminal = Terminal::new(20, 10);
    run(&mut terminal, "long", "abcdefghijklmno\r\n", "0");
    terminal.resize(6, 10);
    let found = seen(&terminal, 0, 10);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].command, Some((0, 6)));
    assert_eq!(output(&terminal, 1).as_deref(), Some("abcdefghijklmno"));
}

#[test]
fn clearing_the_screen_forgets_the_blocks_on_it() {
    let mut terminal = two();
    terminal.screen_mut().erase_in_display(crate::grid::Erase::All);
    assert!(seen(&terminal, 0, 10).is_empty());
    assert_eq!(output(&terminal, 1), None);
}

#[test]
fn an_end_with_no_command_running_changes_nothing_and_a_missing_time_is_unknown() {
    let mut terminal = Terminal::new(20, 5);
    mark(&mut terminal, &["A"]);
    mark(&mut terminal, &["D", "1", "10"]);
    assert_eq!(terminal.blocks.ran(1), None);
    mark(&mut terminal, &["C"]);
    mark(&mut terminal, &["D", "2"]);
    let ran = Ran {
        code: 2,
        millis: None,
    };
    assert_eq!(terminal.blocks.ran(1), Some(ran));
}

#[test]
fn a_duration_is_seconds_with_a_tenth_or_minutes_and_seconds() {
    let spanish = |millis| duration(Language::Spanish, millis);
    assert_eq!(spanish(120), "0,1 s");
    assert_eq!(spanish(41_270), "41,3 s");
    assert_eq!(spanish(125_999), "2 min 5 s");
    assert_eq!(duration(Language::English, 3_800), "3.8 s");
}
