//! What each load repeats.

use std::fmt::Write;

use super::XorShift64;
use crate::metric::Load;

const LINE: usize = 80;
const LONG_LINE: usize = 10_000;
/// The columns a line of the Unicode load takes on screen.
pub(super) const UNICODE_COLUMNS: usize = 40;
const BAR_CELLS: u32 = 40;
const BARS_PER_SCREEN: usize = 10;

/// Pieces with the columns each takes: accented Latin, double width CJK, emoji
/// (some joined with ZWJ) and letters with combining accents.
pub(super) const PIECES: [(&str, usize); 16] = [
    ("a", 1),
    ("é", 1),
    ("ñ", 1),
    ("ü", 1),
    (" ", 1),
    ("Z", 1),
    ("漢", 2),
    ("字", 2),
    ("語", 2),
    ("한", 2),
    ("🚀", 2),
    ("✅", 2),
    ("\u{1F469}\u{200D}\u{1F4BB}", 2),
    ("\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}", 2),
    ("e\u{301}", 1),
    ("n\u{303}", 1),
];

/// Stands in for the recorded build until it is made.
const RECORDING: &str = "   Compiling imperifex v0.1.0\r\n";

/// Adds the unit number `index` of a load.
pub(super) fn push(load: Load, index: usize, rng: &mut XorShift64, out: &mut String) {
    match load {
        Load::Ascii => printable_line(LINE, rng, out),
        Load::LongLines => printable_line(LONG_LINE, rng, out),
        Load::Color => color_line(rng, out),
        Load::Unicode => unicode_line(rng, out),
        Load::Cursor => progress_bar(index, out),
        Load::Cargo => out.push_str(RECORDING),
    }
}

/// One of the printable ASCII characters, from space (32) to `~` (126).
fn printable(rng: &mut XorShift64) -> char {
    char::from(32 + rng.below(95) as u8)
}

fn printable_line(length: usize, rng: &mut XorShift64, out: &mut String) {
    out.extend((0..length).map(|_| printable(rng)));
    out.push_str("\r\n");
}

/// Every character with a colour of its own, in 24 bits.
fn color_line(rng: &mut XorShift64, out: &mut String) {
    for _ in 0..LINE {
        let [r, g, b, ..] = rng.next_u64().to_le_bytes();
        // Writing to a String cannot fail.
        let _ = write!(out, "\x1b[38;2;{r};{g};{b}m{}", printable(rng));
    }
    out.push_str("\x1b[0m\r\n");
}

fn unicode_line(rng: &mut XorShift64, out: &mut String) {
    let mut left = UNICODE_COLUMNS;
    while left > 0 {
        let (piece, columns) = PIECES[rng.below(PIECES.len() as u64) as usize];
        if columns <= left {
            out.push_str(piece);
            left -= columns;
        }
    }
    out.push_str("\r\n");
}

/// A bar that fills in 100 steps, rewriting its line; every tenth bar the
/// screen is cleared.
fn progress_bar(index: usize, out: &mut String) {
    if index > 0 && index.is_multiple_of(BARS_PER_SCREEN) {
        out.push_str("\x1b[H\x1b[2J");
    }
    for percent in 1..=100u32 {
        let filled = (percent * BAR_CELLS / 100) as usize;
        let empty = BAR_CELLS as usize - filled;
        let _ = write!(
            out,
            "\r[{}{}] {percent}%\x1b[K",
            "#".repeat(filled),
            " ".repeat(empty)
        );
    }
    out.push_str("\r\n");
}
