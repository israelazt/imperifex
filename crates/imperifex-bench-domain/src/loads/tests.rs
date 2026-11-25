use super::units::{PIECES, UNICODE_COLUMNS};
use super::*;

const SIZE: usize = 64 * 1024;

fn text(load: Load) -> String {
    String::from_utf8(generate(load, SIZE, 1)).unwrap()
}

#[test]
fn every_load_has_exactly_the_size_asked_for_and_is_utf8() {
    for load in Load::ALL {
        for size in [0, 1, 7, 81, 1000, SIZE] {
            let bytes = generate(load, size, 1);
            assert_eq!(bytes.len(), size, "{} of {size}", load.name());
            assert!(std::str::from_utf8(&bytes).is_ok(), "{}", load.name());
        }
    }
}

/// The bytes of each load are part of the protocol: results taken with other
/// bytes do not compare.
#[test]
fn the_bytes_of_every_load_are_fixed() {
    let hashes = Load::ALL.map(|load| (load.name(), fnv1a64(&generate(load, SIZE, 1))));
    let expected: [(&str, u64); 6] = [
        ("ascii", 0xD265_D834_F09F_3F75),
        ("long_lines", 0x9ED2_99A4_5DD0_8A05),
        ("color", 0x5FB0_563D_F7E4_9950),
        ("unicode", 0x00B2_2632_3E66_64DE),
        ("cursor", 0xE8FF_FACB_B042_07DF),
        ("cargo", 0x13F1_BDCB_04AD_5B87),
    ];
    assert_eq!(hashes, expected, "{hashes:#x?}");
}

#[test]
fn another_seed_gives_other_bytes_where_there_is_chance() {
    for load in [Load::Ascii, Load::LongLines, Load::Color, Load::Unicode] {
        assert_ne!(
            generate(load, SIZE, 1),
            generate(load, SIZE, 2),
            "{}",
            load.name()
        );
    }
}

#[test]
fn ascii_is_lines_of_eighty_printable_characters() {
    let text = text(Load::Ascii);
    let lines: Vec<&str> = text.split("\r\n").collect();
    for line in &lines[..lines.len() - 1] {
        assert_eq!(line.len(), 80, "{line:?}");
    }
    assert!(text
        .bytes()
        .all(|b| (32..=126).contains(&b) || b == b'\r' || b == b'\n'));
}

#[test]
fn long_lines_are_ten_thousand_characters_each() {
    let text = text(Load::LongLines);
    assert_eq!(text.split("\r\n").next().unwrap().len(), 10_000);
}

#[test]
fn every_character_of_color_carries_its_own_and_each_line_resets() {
    let text = text(Load::Color);
    let line = text.split("\r\n").next().unwrap();
    assert_eq!(line.matches("\x1b[38;2;").count(), 80);
    assert!(line.ends_with("\x1b[0m"), "{line:?}");
}

/// The columns of a line, by taking its pieces off one at a time.
fn columns(mut line: &str) -> usize {
    let mut total = 0;
    while !line.is_empty() {
        // The longest first, so that a joined emoji is not read as its parts.
        let (piece, width) = PIECES
            .iter()
            .filter(|(piece, _)| line.starts_with(piece))
            .max_by_key(|(piece, _)| piece.len())
            .unwrap();
        total += width;
        line = &line[piece.len()..];
    }
    total
}

#[test]
fn every_unicode_line_takes_forty_columns() {
    let text = text(Load::Unicode);
    let lines: Vec<&str> = text.split("\r\n").collect();
    for line in &lines[..lines.len() - 1] {
        assert_eq!(columns(line), UNICODE_COLUMNS, "{line}");
    }
    for needed in ["é", "漢", "\u{200D}", "e\u{301}"] {
        assert!(text.contains(needed), "{needed:?}");
    }
}

#[test]
fn the_cursor_load_rewrites_bars_and_clears_every_tenth() {
    let text = String::from_utf8(generate(Load::Cursor, SIZE * 2, 1)).unwrap();
    let first_bar = text.split("\r\n").next().unwrap();
    assert_eq!(first_bar.matches("\x1b[K").count(), 100);
    assert!(first_bar.ends_with("100%\x1b[K"));
    let before_clearing = text.split("\x1b[H\x1b[2J").next().unwrap();
    assert_eq!(before_clearing.matches("\r\n").count(), 10);
    assert!(text.contains("\x1b[H\x1b[2J"));
}

#[test]
fn the_cargo_load_is_a_build() {
    assert!(text(Load::Cargo).contains("Compiling"));
}
