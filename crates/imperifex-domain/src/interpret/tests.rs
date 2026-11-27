use super::*;
use crate::grid::{Attributes, Color, Position, Style};
use crate::sequence::Params;

fn at(row: usize, column: usize) -> Position {
    Position { row, column }
}

/// Feeds a short script: text, `\r`, `\n`, `ESC x`, and `ESC [ … X` with an
/// optional `?` and numbers separated by `;` or `:`. The bytes of a real shell
/// are tested with the parser, in the platform crate.
fn send(terminal: &mut Terminal, script: &str) {
    let mut characters = script.chars();
    while let Some(character) = characters.next() {
        match character {
            '\x1b' => escape(terminal, &mut characters),
            '\r' | '\n' | '\x08' | '\t' => apply(terminal, Sequence::Control(character as u8)),
            other => apply(terminal, Sequence::Print(other)),
        }
    }
}

fn escape(terminal: &mut Terminal, characters: &mut std::str::Chars<'_>) {
    let first = characters.next().unwrap();
    if first != '[' {
        let esc = Sequence::Esc {
            intermediates: &[],
            action: first as u8,
        };
        return apply(terminal, esc);
    }
    let mut body = String::new();
    let action = loop {
        let next = characters.next().unwrap();
        if next.is_ascii_digit() || "?;:".contains(next) {
            body.push(next);
        } else {
            break next;
        }
    };
    let (marker, numbers) = match body.strip_prefix('?') {
        Some(rest) => (&b"?"[..], rest),
        None => (&b""[..], body.as_str()),
    };
    let (mut values, mut joined) = (Vec::new(), 0u32);
    if !numbers.is_empty() {
        let mut after_colon = false;
        for part in numbers.split_inclusive([';', ':']) {
            joined |= u32::from(after_colon) << values.len();
            after_colon = part.ends_with(':');
            values.push(part.trim_end_matches([';', ':']).parse().unwrap_or(0));
        }
        // A separator at the end leaves one more number out: it is a 0.
        if numbers.ends_with([';', ':']) {
            joined |= u32::from(after_colon) << values.len();
            values.push(0);
        }
    }
    let csi = Sequence::Csi {
        params: Params::new(&values, joined),
        intermediates: marker,
        action,
    };
    apply(terminal, csi);
}

fn terminal(columns: u16, rows: u16, script: &str) -> Terminal {
    let mut terminal = Terminal::new(columns, rows);
    send(&mut terminal, script);
    terminal
}

fn text(terminal: &Terminal) -> String {
    terminal.screen().text()
}

fn cursor(terminal: &Terminal) -> Position {
    terminal.screen().cursor()
}

/// A 10 by 5 terminal with the cursor at row 2, column 4.
fn centred() -> Terminal {
    terminal(10, 5, "\x1b[3;5H")
}

/// Five rows reading `a` to `e`.
fn lettered() -> Terminal {
    terminal(10, 5, "a\r\nb\r\nc\r\nd\r\ne")
}

#[test]
fn each_movement_goes_where_it_says() {
    let cases = [
        ("\x1b[2A", at(0, 4)),
        ("\x1b[2B", at(4, 4)),
        ("\x1b[2e", at(4, 4)),
        ("\x1b[3C", at(2, 7)),
        ("\x1b[3a", at(2, 7)),
        ("\x1b[3D", at(2, 1)),
        ("\x1b[1E", at(3, 0)),
        ("\x1b[1F", at(1, 0)),
        ("\x1b[7G", at(2, 6)),
        ("\x1b[7`", at(2, 6)),
        ("\x1b[5d", at(4, 4)),
        ("\x1b[2;3H", at(1, 2)),
        ("\x1b[4;9f", at(3, 8)),
        ("\x1b[0H", at(0, 0)),
    ];
    for (script, expected) in cases {
        let mut terminal = centred();
        send(&mut terminal, script);
        assert_eq!(cursor(&terminal), expected, "{script:?}");
    }
}

#[test]
fn nothing_and_0_move_one_step() {
    for script in ["\x1b[C", "\x1b[0C"] {
        let mut terminal = centred();
        send(&mut terminal, script);
        assert_eq!(cursor(&terminal), at(2, 5), "{script:?}");
    }
}

#[test]
fn a_huge_number_stays_inside_the_grid() {
    for action in "ABCDEFGHdefa`JKX@PLMSTrsu".chars() {
        let mut terminal = centred();
        send(&mut terminal, &format!("\x1b[65535;65535{action}"));
        let cursor = cursor(&terminal);
        assert!(cursor.row < 5 && cursor.column < 10, "{action}");
    }
}

#[test]
fn the_controls_move_the_cursor() {
    let mut terminal = terminal(20, 5, "ab");
    let moves = [
        (0x08, at(0, 1)),
        (0x09, at(0, 8)),
        (0x0A, at(1, 8)),
        (0x0B, at(2, 8)),
        (0x0C, at(3, 8)),
        (0x0D, at(3, 0)),
        // The bell does nothing to the screen.
        (0x07, at(3, 0)),
    ];
    for (byte, expected) in moves {
        apply(&mut terminal, Sequence::Control(byte));
        assert_eq!(cursor(&terminal), expected, "{byte:#x}");
    }
}

#[test]
fn erasing_follows_its_parameter() {
    for (script, expected) in [
        ("\x1b[0K", "aaaa\nbb\ncccc"),
        ("\x1b[1K", "aaaa\n   b\ncccc"),
        ("\x1b[2K", "aaaa\n\ncccc"),
        ("\x1b[0J", "aaaa\nbb\n"),
        ("\x1b[1J", "\n   b\ncccc"),
        ("\x1b[2J", "\n\n"),
        // The history is not part of the visible screen.
        ("\x1b[3J", "aaaa\nbbbb\ncccc"),
        ("\x1b[1X", "aaaa\nbb b\ncccc"),
    ] {
        let mut terminal = terminal(10, 3, "aaaa\r\nbbbb\r\ncccc\x1b[2;3H");
        send(&mut terminal, script);
        assert_eq!(text(&terminal), expected, "{script:?}");
    }
}

#[test]
fn characters_are_inserted_and_deleted_at_the_cursor() {
    for (script, expected) in [
        ("\x1b[2@", "ab  cdef"),
        ("\x1b[2P", "abef"),
        ("\x1b[99@", "ab"),
        ("\x1b[99P", "ab"),
        ("\x1b[4hXY", "abXYcdef"),
        ("\x1b[4h\x1b[4lXY", "abXYef"),
    ] {
        let mut terminal = terminal(8, 1, "abcdef\x1b[3G");
        send(&mut terminal, script);
        assert_eq!(text(&terminal), expected, "{script:?}");
    }
}

#[test]
fn inserting_and_deleting_never_leave_half_a_wide_character() {
    for (script, expected) in [
        // At the cursor.
        ("\x1b[3G\x1b[@", "a   b"),
        ("\x1b[3G\x1b[P", "a b"),
        // At the end of the row: the one pushed half out goes whole.
        ("\x1b[1G\x1b[3@", "   a"),
    ] {
        let mut terminal = terminal(5, 1, "a漢b");
        send(&mut terminal, script);
        let shown = text(&terminal);
        assert_eq!(shown.trim_end(), expected.trim_end(), "{script:?}");
    }
}

#[test]
fn only_the_scroll_region_moves() {
    let cases = [
        // A line feed on the bottom of the region.
        ("\x1b[2;4r\x1b[4;1H\n", "a\nc\nd\n\ne"),
        // A reverse line feed on its top.
        ("\x1b[2;4r\x1b[2;1H\x1bM", "a\n\nb\nc\ne"),
        ("\x1b[2;4r\x1b[S", "a\nc\nd\n\ne"),
        ("\x1b[2;4r\x1b[T", "a\n\nb\nc\ne"),
        ("\x1b[2;4r\x1b[3;1H\x1b[L", "a\nb\n\nc\ne"),
        ("\x1b[2;4r\x1b[3;1H\x1b[M", "a\nb\nd\n\ne"),
        // Inserting and deleting outside the region does nothing.
        ("\x1b[2;4r\x1b[5;1H\x1b[L", "a\nb\nc\nd\ne"),
        ("\x1b[2;4r\x1b[1;1H\x1b[M", "a\nb\nc\nd\ne"),
        // On the last row, below the region, nothing scrolls.
        ("\x1b[2;4r\x1b[5;1H\n", "a\nb\nc\nd\ne"),
        // A region of one row is not one: the whole screen scrolls.
        ("\x1b[3;3r\x1b[5;1H\n", "b\nc\nd\ne\n"),
        // No parameters give the whole screen back.
        ("\x1b[2;4r\x1b[r\x1b[5;1H\n", "b\nc\nd\ne\n"),
    ];
    for (script, expected) in cases {
        let mut terminal = lettered();
        send(&mut terminal, script);
        assert_eq!(text(&terminal), expected, "{script:?}");
    }
}

#[test]
fn going_up_and_down_stops_at_the_edges_of_the_region_from_inside() {
    let mut terminal = terminal(10, 9, "\x1b[3;6r\x1b[4;1H\x1b[99A");
    assert_eq!(cursor(&terminal), at(2, 0));
    send(&mut terminal, "\x1b[99B");
    assert_eq!(cursor(&terminal), at(5, 0));
    send(&mut terminal, "\x1b[9;1H\x1b[99A");
    assert_eq!(cursor(&terminal), at(2, 0));
    send(&mut terminal, "\x1b[1;1H\x1b[99A");
    assert_eq!(cursor(&terminal), at(0, 0));
}

#[test]
fn in_origin_mode_rows_count_from_the_top_of_the_region() {
    let mut terminal = terminal(10, 9, "\x1b[3;6r\x1b[?6h");
    assert_eq!(cursor(&terminal), at(2, 0));
    send(&mut terminal, "\x1b[2;3H");
    assert_eq!(cursor(&terminal), at(3, 2));
    send(&mut terminal, "\x1b[99;1H");
    assert_eq!(cursor(&terminal), at(5, 0));
    send(&mut terminal, "\x1b[?6l");
    assert_eq!(cursor(&terminal), at(0, 0));
}

#[test]
fn the_saved_cursor_comes_back_with_its_style() {
    for (save, restore) in [("\x1b7", "\x1b8"), ("\x1b[s", "\x1b[u")] {
        let mut terminal = centred();
        send(
            &mut terminal,
            &format!("\x1b[31m{save}\x1b[0m\x1b[1;1H{restore}"),
        );
        assert_eq!(cursor(&terminal), at(2, 4));
        assert_eq!(terminal.screen().pen.foreground, Color::Indexed(1));
    }
}

#[test]
fn restoring_with_nothing_saved_goes_to_the_start() {
    let mut terminal = centred();
    send(&mut terminal, "\x1b[1m\x1b8");
    assert_eq!(cursor(&terminal), at(0, 0));
    assert_eq!(terminal.screen().pen, Style::default());
}

#[test]
fn a_reset_leaves_the_terminal_as_new() {
    let mut terminal = terminal(10, 3, "hola\x1b[31m\x1b[?25l\x1b[?1049h\x1bc");
    assert_eq!(text(&terminal), "\n\n");
    assert!(!terminal.on_alternate_screen());
    assert!(terminal.modes.cursor_visible);
    send(&mut terminal, "x");
    assert_eq!(
        terminal.screen().cell(0, 0).unwrap().style(),
        Style::default()
    );
}

#[test]
fn the_alternate_screen_leaves_the_main_one_as_it_was() {
    let mut terminal = terminal(10, 3, "uno\r\ndos\x1b[?1049h");
    assert!(terminal.on_alternate_screen());
    assert_eq!(text(&terminal), "\n\n");
    // It starts where the cursor was.
    assert_eq!(cursor(&terminal), at(1, 3));
    send(&mut terminal, "\x1b[1;1Hvim\x1b[?1049l");
    assert!(!terminal.on_alternate_screen());
    assert_eq!(text(&terminal), "uno\ndos\n");
    assert_eq!(cursor(&terminal), at(1, 3));
}

#[test]
fn the_alternate_screen_starts_empty_each_time_and_entering_twice_is_once() {
    for mode in [47, 1047, 1049] {
        let mut terminal = terminal(10, 2, "principal");
        let enter = format!("\x1b[?{mode}h");
        let leave = format!("\x1b[?{mode}l");
        send(&mut terminal, &format!("{enter}viejo{leave}{enter}"));
        assert_eq!(text(&terminal), "\n", "{mode}");
        send(&mut terminal, &format!("{enter}{leave}"));
        assert_eq!(text(&terminal), "principal\n", "{mode}");
    }
}

#[test]
fn each_screen_has_its_own_scroll_region() {
    let mut terminal = lettered();
    send(&mut terminal, "\x1b[?1049h\x1b[2;3r\x1b[?1049l\x1b[5;1H\n");
    assert_eq!(text(&terminal), "b\nc\nd\ne\n");
}

#[test]
fn modes_the_window_follows_are_kept() {
    let mut terminal = terminal(10, 2, "\x1b[?1;25;2004h");
    assert!(terminal.modes.application_cursor_keys);
    assert!(terminal.modes.cursor_visible && terminal.modes.bracketed_paste);
    send(&mut terminal, "\x1b[?1;25;2004l");
    assert!(!terminal.modes.application_cursor_keys);
    assert!(!terminal.modes.cursor_visible && !terminal.modes.bracketed_paste);
}

#[test]
fn without_autowrap_the_last_column_is_written_over() {
    let terminal = terminal(4, 2, "\x1b[?7labcdef");
    assert_eq!(text(&terminal), "abcf\n");
    assert_eq!(cursor(&terminal), at(0, 3));
}

/// The pen after one SGR sequence.
fn pen(body: &str) -> Style {
    terminal(10, 1, &format!("\x1b[{body}m")).screen().pen
}

#[test]
fn each_attribute_goes_on_and_off() {
    let cases = [
        ("1", "22", Attributes::BOLD),
        ("2", "22", Attributes::DIM),
        ("3", "23", Attributes::ITALIC),
        ("4", "24", Attributes::UNDERLINE),
        ("4:3", "4:0", Attributes::UNDERLINE),
        ("21", "24", Attributes::UNDERLINE),
        ("5", "25", Attributes::BLINK),
        ("7", "27", Attributes::INVERSE),
        ("8", "28", Attributes::HIDDEN),
        ("9", "29", Attributes::STRIKE),
    ];
    for (on, off, attribute) in cases {
        assert!(pen(on).attributes.has(attribute), "{on}");
        assert_eq!(pen(&format!("{on};{off}")), Style::default(), "{on};{off}");
    }
}

#[test]
fn the_numbered_colours_go_to_letter_and_background() {
    let cases = [("30;40", 0), ("37;47", 7), ("90;100", 8), ("97;107", 15)];
    for (body, index) in cases {
        let style = pen(body);
        assert_eq!(style.foreground, Color::Indexed(index), "{body}");
        assert_eq!(style.background, Color::Indexed(index), "{body}");
    }
    assert_eq!(pen("31;41;39;49"), Style::default());
}

#[test]
fn long_colours_are_read_in_both_forms_and_what_follows_still_counts() {
    for body in ["38;5;208;1", "38:5:208;1", "38;2;1;2;3;1", "38:2::1:2:3;1"] {
        let style = pen(body);
        let expected = if body.contains("208") {
            Color::Indexed(208)
        } else {
            Color::Rgb(1, 2, 3)
        };
        assert_eq!(style.foreground, expected, "{body}");
        assert!(style.attributes.has(Attributes::BOLD), "{body}");
    }
}

#[test]
fn a_colour_that_is_wrong_or_cut_changes_nothing() {
    for body in [
        "38;5;999",
        "38;2;1;2",
        "38",
        "48;9",
        "38:2:1",
        "58;2;1;2;3",
        "58:5:1",
    ] {
        assert_eq!(pen(body), Style::default(), "{body}");
    }
}

#[test]
fn no_parameters_and_0_take_all_style_off() {
    assert_eq!(pen("1;31;"), Style::default());
    let terminal = terminal(10, 1, "\x1b[1;31m\x1b[m");
    assert_eq!(terminal.screen().pen, Style::default());
}

#[test]
fn text_keeps_its_style_and_what_is_emptied_keeps_the_background() {
    let terminal = terminal(5, 2, "\x1b[1;31;44mab\x1b[K");
    let style = |column| terminal.screen().cell(0, column).unwrap().style();
    assert_eq!(style(0).foreground, Color::Indexed(1));
    assert!(style(0).attributes.has(Attributes::BOLD));
    let emptied = Style {
        background: Color::Indexed(4),
        ..Style::default()
    };
    assert_eq!(style(3), emptied);
}

#[test]
fn a_marker_changes_the_meaning_so_the_sequence_is_ignored() {
    let mut terminal = centred();
    for intermediates in [&b">"[..], b" ", b"!"] {
        apply(
            &mut terminal,
            Sequence::Csi {
                params: Params::from(&[2][..]),
                intermediates,
                action: 'J',
            },
        );
    }
    apply(
        &mut terminal,
        Sequence::Esc {
            intermediates: b"(",
            action: b'E',
        },
    );
    send(&mut terminal, "\x1b[?2J");
    assert_eq!(cursor(&terminal), at(2, 4));
}

#[test]
fn what_is_not_known_changes_nothing() {
    let mut terminal = centred();
    send(&mut terminal, "x");
    let before = (text(&terminal), cursor(&terminal), terminal.modes);
    send(
        &mut terminal,
        "\x1b[z\x1b[?9001h\x1b[?1004h\x1b[1;2;3;4;5T\x1bZ",
    );
    apply(&mut terminal, Sequence::Osc(&[b"0", b"titulo"]));
    assert_eq!((text(&terminal), cursor(&terminal), terminal.modes), before);
}
