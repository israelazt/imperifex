use imperifex_domain::grid::{Grid, Position, Span};
use imperifex_domain::interpret::apply;

use super::*;

/// The grid after feeding the chunks, one after the other.
fn shown<'a>(columns: u16, rows: u16, chunks: impl IntoIterator<Item = &'a [u8]>) -> Grid {
    let mut grid = Grid::new(columns, rows);
    let mut parser = VteParser::new();
    for chunk in chunks {
        parser.parse(chunk, &mut |sequence| apply(&mut grid, sequence));
    }
    grid
}

fn state(grid: &Grid) -> (String, Position) {
    (grid.text(), grid.cursor())
}

/// How `cmd` started under ConPTY in the T1.1 probe, with a made-up folder.
const CMD_START: &str = "\x1b[?9001h\x1b[?1004h\x1b[?25l\x1b[2J\x1b[m\x1b[H\
    Microsoft Windows [Versión 10.0.26200.9974]\x1b]0;C:\\WINDOWS\\SYSTEM32\\cmd.exe\x07\
    \x1b[?25h\x1b[?25l\r\n(c) Microsoft Corporation. Todos los derechos reservados.\
    \x1b[4;1HC:\\proyecto>\x1b[?25h";

#[test]
fn the_start_of_cmd_gives_the_screen_it_draws() {
    let grid = shown(80, 25, [CMD_START.as_bytes()]);
    assert_eq!(
        grid.text().trim_end(),
        "Microsoft Windows [Versión 10.0.26200.9974]\n\
         (c) Microsoft Corporation. Todos los derechos reservados.\n\
         \n\
         C:\\proyecto>"
    );
    assert_eq!(grid.cursor(), Position { row: 3, column: 12 });
}

#[test]
fn cutting_the_input_at_any_byte_changes_nothing() {
    let bytes = format!("{CMD_START}漢e\u{301}\x1b[3D\x1b[2X\x1b]133;A\x1b\\z");
    let bytes = bytes.as_bytes();
    let whole = state(&shown(80, 25, [bytes]));
    for cut in 0..=bytes.len() {
        let parts = shown(80, 25, [&bytes[..cut], &bytes[cut..]]);
        assert_eq!(state(&parts), whole, "cut at {cut}");
    }
    assert_eq!(state(&shown(80, 25, bytes.chunks(1))), whole);
}

#[test]
fn bytes_that_are_not_text_show_a_replacement_and_go_on() {
    let grid = shown(10, 1, [&b"a\xff\xfeb\xc3"[..], &b"\x1b[Cc"[..]]);
    assert!(grid.row_text(0).starts_with("a\u{fffd}\u{fffd}b"));
    assert!(grid.row_text(0).ends_with('c'));
}

#[test]
fn a_csi_with_too_many_parameters_is_not_passed_on() {
    let long = format!("ab\x1b[{}Hc", "1;".repeat(100));
    assert_eq!(shown(10, 2, [long.as_bytes()]).text(), "abc\n");
}

/// Everything the parser hands over for `bytes`, as the OSC lengths.
fn osc_lengths(bytes: &[u8]) -> Vec<usize> {
    let mut lengths = Vec::new();
    VteParser::new().parse(bytes, &mut |sequence| {
        if let Sequence::Osc(parts) = sequence {
            lengths.push(parts.iter().map(|part| part.len()).sum());
        }
    });
    lengths
}

/// Guards the missing `std` feature of `vte`: with it, this keeps the whole
/// megabyte.
#[test]
fn a_huge_osc_arrives_cut_to_1024_bytes() {
    let mut bytes = b"\x1b]0;".to_vec();
    bytes.extend(vec![b'x'; 1 << 20]);
    bytes.push(0x07);
    assert_eq!(osc_lengths(&bytes), [1024]);
}

/// A fixed sequence of numbers: the same run every time.
struct Numbers(u64);

impl Numbers {
    fn below(&mut self, limit: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        (self.0 >> 33) as usize % limit
    }
}

/// Random bytes, with many of those that start and shape sequences.
fn noise(length: usize, seed: u64) -> Vec<u8> {
    const SHAPING: &[u8] = b"\x1b\x1b[[]P;;:?0123456789ABCDEFGHJKXdfm\x07\x08\t\r\n\\\x18";
    let mut numbers = Numbers(seed);
    (0..length)
        .map(|_| match numbers.below(4) {
            0 => numbers.below(256) as u8,
            _ => SHAPING[numbers.below(SHAPING.len())],
        })
        .collect()
}

fn has_no_half_character(grid: &Grid) -> bool {
    (0..grid.rows()).all(|row| {
        (0..grid.columns()).all(|column| {
            let here = grid.cell(row, column).map(|cell| cell.span());
            let next = grid.cell(row, column + 1).map(|cell| cell.span());
            (here == Some(Span::Double)) == (next == Some(Span::Continuation))
        })
    })
}

#[test]
fn noise_breaks_nothing() {
    for seed in 0..20 {
        let bytes = noise(50_000, seed);
        let grid = shown(13, 5, bytes.chunks(1 + seed as usize * 7));
        assert!(has_no_half_character(&grid), "seed {seed}");
        let cursor = grid.cursor();
        assert!(cursor.row < 5 && cursor.column < 13, "seed {seed}");
    }
}
