use super::*;

fn grid(columns: u16, rows: u16, written: &str) -> Grid {
    let mut grid = Grid::new(columns, rows);
    write(&mut grid, written);
    grid
}

/// Writes text, with `\r`, `\n`, `\t` and `\x08` as their movements.
fn write(grid: &mut Grid, text: &str) {
    for character in text.chars() {
        match character {
            '\r' => grid.carriage_return(),
            '\n' => grid.line_feed(),
            '\t' => grid.tab(),
            '\x08' => grid.backspace(),
            other => grid.print(other),
        }
    }
}

fn at(row: usize, column: usize) -> Position {
    Position { row, column }
}

fn span(grid: &Grid, column: usize) -> Span {
    grid.cell(0, column).unwrap().span()
}

#[test]
fn a_new_grid_is_empty_with_the_cursor_at_the_start() {
    let grid = Grid::new(10, 3);
    assert_eq!((grid.columns(), grid.rows()), (10, 3));
    assert_eq!(grid.cursor(), at(0, 0));
    assert_eq!(grid.text(), "\n\n");
}

#[test]
fn the_size_is_kept_between_1_and_1000() {
    let small = Grid::new(0, 0);
    assert_eq!((small.columns(), small.rows()), (1, 1));
    let big = Grid::new(u16::MAX, u16::MAX);
    assert_eq!((big.columns(), big.rows()), (1000, 1000));
}

#[test]
fn text_is_written_where_the_cursor_is() {
    let grid = grid(10, 2, "hola\r\nmundo");
    assert_eq!(grid.text(), "hola\nmundo");
    assert_eq!(grid.cursor(), at(1, 5));
}

#[test]
fn control_characters_write_nothing() {
    let grid = grid(10, 1, "a\x07\x1bb");
    assert_eq!(grid.text(), "ab");
}

#[test]
fn the_wrap_after_the_last_column_waits_for_the_next_character() {
    let mut grid = grid(4, 2, "abcd");
    assert_eq!(grid.cursor(), at(0, 3));
    grid.print('e');
    assert_eq!(grid.text(), "abcd\ne");
    assert_eq!(grid.cursor(), at(1, 1));
}

#[test]
fn moving_the_cursor_forgets_the_pending_wrap() {
    let grid = grid(4, 2, "abcd\rx");
    assert_eq!(grid.text(), "xbcd\n");
    let mut other = super::tests::grid(4, 2, "abcd");
    other.backspace();
    other.print('y');
    assert_eq!(other.text(), "abyd\n");
}

#[test]
fn a_line_feed_keeps_the_column() {
    let grid = grid(10, 2, "ab\ncd");
    assert_eq!(grid.text(), "ab\n  cd");
}

#[test]
fn the_last_row_scrolls_and_the_top_one_is_lost() {
    let grid = grid(10, 2, "uno\r\ndos\r\ntres");
    assert_eq!(grid.text(), "dos\ntres");
    assert_eq!(grid.cursor(), at(1, 4));
}

#[test]
fn backspace_stops_at_the_first_column() {
    let grid = grid(10, 1, "a\x08\x08\x08b");
    assert_eq!(grid.text(), "b");
}

#[test]
fn a_tab_goes_to_the_next_multiple_of_8_and_not_past_the_last_column() {
    let mut grid = grid(20, 1, "ab\t");
    assert_eq!(grid.cursor(), at(0, 8));
    grid.tab();
    grid.tab();
    assert_eq!(grid.cursor(), at(0, 19));
}

#[test]
fn moving_is_kept_inside_the_grid() {
    let mut grid = Grid::new(10, 3);
    grid.move_to(99, 99);
    assert_eq!(grid.cursor(), at(2, 9));
}

#[test]
fn a_wide_character_takes_two_columns() {
    let grid = grid(10, 1, "a漢b");
    assert_eq!(grid.text(), "a漢b");
    assert_eq!(grid.cursor(), at(0, 4));
    assert_eq!(span(&grid, 1), Span::Double);
    assert_eq!(span(&grid, 2), Span::Continuation);
}

#[test]
fn a_wide_character_that_ends_the_row_leaves_the_wrap_pending() {
    let grid = grid(4, 2, "ab漢c");
    assert_eq!(grid.text(), "ab漢\nc");
}

#[test]
fn a_wide_character_that_does_not_fit_wraps_first() {
    let grid = grid(4, 2, "abc漢");
    assert_eq!(grid.text(), "abc\n漢");
}

#[test]
fn a_wide_character_is_dropped_in_a_grid_of_one_column() {
    let grid = grid(1, 2, "漢a");
    assert_eq!(grid.text(), "a\n");
}

#[test]
fn writing_over_either_half_of_a_wide_character_removes_both() {
    let mut first = grid(10, 1, "漢z");
    first.move_to(0, 0);
    first.print('a');
    assert_eq!(first.text(), "a z");

    let mut second = grid(10, 1, "漢z");
    second.move_to(0, 1);
    second.print('b');
    assert_eq!(second.text(), " bz");
}

#[test]
fn a_wide_character_written_across_two_others_removes_both_whole() {
    let mut grid = grid(10, 1, "漢字z");
    grid.move_to(0, 1);
    grid.print('語');
    assert_eq!(grid.text(), " 語 z");
}

#[test]
fn a_combining_mark_joins_the_character_before_it() {
    let grid = grid(10, 1, "e\u{301}x");
    assert_eq!(grid.row_text(0), "e\u{301}x");
    assert_eq!(grid.cursor(), at(0, 2));
}

#[test]
fn a_mark_after_a_wide_character_or_at_the_end_of_the_row_finds_it() {
    let wide = grid(10, 1, "漢\u{301}");
    assert_eq!(wide.row_text(0), "漢\u{301}");
    let last = grid(2, 2, "ab\u{301}");
    assert_eq!(last.text(), "ab\u{301}\n");
}

#[test]
fn a_mark_with_nothing_before_it_is_dropped() {
    let grid = grid(10, 1, "\u{301}a");
    assert_eq!(grid.row_text(0), "a");
}

#[test]
fn a_cell_keeps_at_most_8_marks() {
    let grid = grid(10, 1, &format!("a{}", "\u{301}".repeat(100)));
    assert_eq!(grid.row_text(0).chars().count(), 9);
}

#[test]
fn erasing_in_a_line_includes_the_cursor_and_does_not_move_it() {
    let cases = [
        (Erase::ToEnd, "ab"),
        (Erase::ToStart, "   de"),
        (Erase::All, ""),
    ];
    for (what, expected) in cases {
        let mut grid = grid(10, 1, "abcde");
        grid.move_to(0, 2);
        grid.erase_in_line(what);
        assert_eq!(grid.text(), expected, "{what:?}");
        assert_eq!(grid.cursor(), at(0, 2));
    }
}

#[test]
fn erasing_in_the_display_covers_the_rows_on_that_side() {
    let cases = [
        (Erase::ToEnd, "aaa\nb\n"),
        (Erase::ToStart, "\n  b\nccc"),
        (Erase::All, "\n\n"),
    ];
    for (what, expected) in cases {
        let mut grid = grid(5, 3, "aaa\r\nbbb\r\nccc");
        grid.move_to(1, 1);
        grid.erase_in_display(what);
        assert_eq!(grid.text(), expected, "{what:?}");
    }
}

#[test]
fn erasing_never_leaves_half_a_wide_character() {
    let mut to_end = grid(10, 1, "a漢b");
    to_end.move_to(0, 2);
    to_end.erase_in_line(Erase::ToEnd);
    assert_eq!(to_end.text(), "a");

    let mut to_start = grid(10, 1, "a漢b");
    to_start.move_to(0, 1);
    to_start.erase_in_line(Erase::ToStart);
    assert_eq!(to_start.text(), "   b");
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

fn do_one(grid: &mut Grid, numbers: &mut Numbers) {
    const CHARACTERS: [char; 5] = ['a', ' ', '漢', '\u{301}', '\u{7}'];
    const ERASES: [Erase; 3] = [Erase::ToEnd, Erase::ToStart, Erase::All];
    match numbers.below(28) {
        0 => grid.carriage_return(),
        1 => grid.line_feed(),
        2 => grid.backspace(),
        3 => grid.tab(),
        4 => grid.move_to(numbers.below(9), numbers.below(9)),
        5 => grid.erase_in_line(ERASES[numbers.below(3)]),
        6 => grid.erase_in_display(ERASES[numbers.below(3)]),
        7 => grid.insert_chars(numbers.below(9)),
        8 => grid.delete_chars(numbers.below(9)),
        9 => grid.erase_chars(numbers.below(9)),
        10 => grid.insert_lines(numbers.below(9)),
        11 => grid.delete_lines(numbers.below(9)),
        12 => grid.set_region(numbers.below(9), numbers.below(9)),
        13 => grid.reverse_line_feed(),
        14 => grid.scroll_down(numbers.below(9)),
        15 => grid.scroll_up(numbers.below(9)),
        16 => grid.insert = numbers.below(2) == 0,
        17 => grid.autowrap = numbers.below(2) == 0,
        18 => grid.set_origin(numbers.below(2) == 0),
        19 => grid.save_cursor(),
        20 => grid.restore_cursor(),
        _ => grid.print(CHARACTERS[numbers.below(CHARACTERS.len())]),
    }
}

fn is_whole(grid: &Grid, row: usize) -> bool {
    (0..grid.columns()).all(|column| {
        let here = grid.cell(row, column).unwrap().span();
        let next = grid.cell(row, column + 1).map(Cell::span);
        let wants_second = here == Span::Double;
        wants_second == (next == Some(Span::Continuation))
            && !(column == 0 && here == Span::Continuation)
    })
}

#[test]
fn no_mix_of_operations_breaks_a_wide_character_or_loses_the_cursor() {
    for (columns, rows) in [(1, 1), (2, 2), (7, 4)] {
        let mut grid = Grid::new(columns, rows);
        let mut numbers = Numbers(1);
        for step in 0..20_000 {
            do_one(&mut grid, &mut numbers);
            assert!((0..grid.rows()).all(|r| is_whole(&grid, r)), "step {step}");
            let cursor = grid.cursor();
            assert!(cursor.row < grid.rows() && cursor.column < grid.columns());
        }
    }
}
