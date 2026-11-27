use super::*;
use crate::grid::{Color, Erase};

/// Text is written as it comes; `\n` goes to the start of the next row.
fn write(grid: &mut Grid, text: &str) {
    for character in text.chars() {
        if character == '\n' {
            grid.carriage_return();
            grid.line_feed();
        } else {
            grid.print(character);
        }
    }
}

fn grid(columns: u16, rows: u16, text: &str) -> Grid {
    let mut grid = Grid::with_history(columns, rows, 100);
    write(&mut grid, text);
    grid
}

fn history(grid: &Grid) -> Vec<&str> {
    (0..grid.history_len())
        .filter_map(|index| grid.history_text(index))
        .collect()
}

fn at(row: usize, column: usize) -> Position {
    Position { row, column }
}

#[test]
fn rows_that_leave_through_the_top_go_to_the_history() {
    let grid = grid(10, 2, "uno\ndos\ntres\ncuatro");
    assert_eq!(history(&grid), ["uno", "dos"]);
    assert_eq!(grid.text(), "tres\ncuatro");
    assert_eq!(grid.text_with_history(), "uno\ndos\ntres\ncuatro");
}

#[test]
fn past_its_limit_the_history_forgets_the_oldest() {
    let mut grid = Grid::with_history(10, 1, 2);
    write(&mut grid, "a\nb\nc\nd");
    assert_eq!(history(&grid), ["b", "c"]);
}

#[test]
fn without_a_limit_nothing_is_kept() {
    let mut grid = Grid::new(10, 1);
    write(&mut grid, "a\nb\nc");
    assert_eq!(grid.history_len(), 0);
}

#[test]
fn only_what_scrolls_out_through_the_top_is_kept() {
    let mut lower = grid(10, 3, "a\nb\nc");
    lower.set_region(1, 2);
    lower.move_to(2, 0);
    lower.line_feed();
    assert_eq!(lower.history_len(), 0);

    let mut deleted = grid(10, 3, "a\nb\nc");
    deleted.move_to(0, 0);
    deleted.delete_lines(1);
    assert_eq!(deleted.history_len(), 0);

    // A fixed line at the bottom does not stop what leaves at the top.
    let mut status = grid(10, 3, "a\nb\nc");
    status.set_region(0, 1);
    status.move_to(1, 0);
    status.line_feed();
    assert_eq!(history(&status), ["a"]);
}

#[test]
fn a_huge_scroll_keeps_each_row_once() {
    let mut grid = grid(10, 2, "a\nb");
    grid.scroll_up(usize::MAX);
    assert_eq!(history(&grid), ["a", "b"]);
}

#[test]
fn erasing_the_screen_leaves_the_history_and_clearing_it_does_not() {
    let mut grid = grid(10, 1, "a\nb");
    grid.erase_in_display(Erase::All);
    assert_eq!(history(&grid), ["a"]);
    grid.clear_history();
    assert_eq!(grid.history_len(), 0);
}

#[test]
fn a_line_comes_back_from_the_history_with_the_same_cells() {
    let mut grid = Grid::with_history(12, 2, 10);
    write(&mut grid, "a漢e\u{301} ");
    grid.pen.foreground = Color::Rgb(1, 2, 3);
    write(&mut grid, "x漢");
    grid.pen.background = Color::Indexed(4);
    write(&mut grid, "  ");
    let written: Vec<_> = (0..12).map(|c| grid.cell(0, c).cloned().unwrap()).collect();
    write(&mut grid, "\n\n");
    let kept = grid.history_cells(0).unwrap();
    // The coloured spaces at the end stay; the untouched ones are not kept.
    assert_eq!(kept, written[..kept.len()]);
    assert_eq!(kept.len(), 10);
    assert!(written[kept.len()..].iter().all(|c| *c == Cell::default()));
}

#[test]
fn narrowing_cuts_long_lines_and_widening_joins_them_again() {
    let mut grid = grid(10, 4, "abcdefghij1234\nfin");
    assert_eq!(grid.text(), "abcdefghij\n1234\nfin\n");
    grid.resize(5, 4);
    assert_eq!(grid.text_with_history(), "abcde\nfghij\n1234\nfin");
    grid.resize(20, 4);
    assert_eq!(grid.text(), "abcdefghij1234\nfin\n\n");
}

#[test]
fn lines_that_ended_by_themselves_are_not_joined() {
    let mut grid = grid(5, 4, "abcde\nfg");
    grid.resize(20, 4);
    assert_eq!(grid.text(), "abcde\nfg\n\n");
}

#[test]
fn a_wide_character_that_wrapped_early_joins_without_a_space() {
    let mut grid = grid(4, 3, "abc漢d");
    assert_eq!(grid.text(), "abc\n漢d\n");
    grid.resize(10, 3);
    assert_eq!(grid.text(), "abc漢d\n\n");
    grid.resize(4, 3);
    assert_eq!(grid.text(), "abc\n漢d\n");
}

/// Found by the random test: the rows cut into the history lost their empty
/// last cell, and the next change of width took a letter in its place.
#[test]
fn a_row_cut_into_the_history_before_a_wide_character_loses_nothing() {
    let mut grid = grid(10, 1, "abc漢defgh");
    grid.resize(4, 1);
    grid.resize(7, 1);
    grid.resize(20, 1);
    assert_eq!(grid.text_with_history(), "abc漢defgh");
}

#[test]
fn a_line_that_goes_on_from_the_history_into_the_screen_is_joined() {
    let mut grid = grid(4, 1, "abcdefgh");
    assert_eq!(history(&grid), ["abcd"]);
    grid.resize(8, 1);
    assert_eq!((grid.history_len(), grid.text().as_str()), (0, "abcdefgh"));
}

#[test]
fn writing_over_the_end_of_a_row_ends_its_line_there() {
    let mut grid = grid(4, 3, "abcdef");
    grid.move_to(0, 3);
    grid.print('X');
    grid.move_to(1, 2);
    grid.resize(10, 3);
    assert_eq!(grid.text(), "abcX\nef\n");
}

#[test]
fn the_cursor_stays_on_its_text() {
    let mut grid = grid(10, 3, "abcdefghijklm");
    grid.move_to(1, 1);
    grid.resize(4, 3);
    // It was on the `l`, which is now the last of the third cut.
    let cursor = grid.cursor();
    let under = grid.cell(cursor.row, cursor.column).cloned().unwrap();
    let mut shown = String::new();
    under.push_to(&mut shown);
    assert_eq!(shown, "l");
}

#[test]
fn the_cursor_after_the_text_keeps_its_distance() {
    let mut grid = grid(20, 2, "C:\\> ");
    grid.resize(10, 2);
    assert_eq!(grid.cursor(), at(0, 5));
}

#[test]
fn a_pending_wrap_is_still_pending() {
    let mut grid = grid(8, 2, "abcd");
    grid.resize(4, 2);
    assert_eq!(grid.cursor(), at(0, 3));
    grid.print('e');
    assert_eq!(grid.text(), "abcd\ne");
}

#[test]
fn fewer_rows_send_the_top_ones_to_the_history_and_more_bring_them_back() {
    let mut grid = grid(10, 3, "a\nb\nc");
    grid.resize(10, 1);
    assert_eq!(history(&grid), ["a", "b"]);
    assert_eq!(grid.text(), "c");
    grid.resize(10, 4);
    assert_eq!(grid.history_len(), 0);
    assert_eq!(grid.text(), "a\nb\nc\n");
    assert_eq!(grid.cursor(), at(2, 1));
}

#[test]
fn empty_rows_under_the_cursor_do_not_push_text_into_the_history() {
    let mut grid = grid(10, 5, "a");
    grid.resize(10, 2);
    assert_eq!((grid.history_len(), grid.text().as_str()), (0, "a\n"));
}

#[test]
fn with_the_cursor_high_up_it_stays_on_the_screen() {
    let mut grid = grid(10, 4, "a\nb\nc\nd");
    grid.move_to(0, 0);
    grid.resize(10, 2);
    assert_eq!(grid.cursor(), at(0, 0));
    assert_eq!(grid.text(), "a\nb");
}

#[test]
fn narrowing_respects_the_limit_of_the_history() {
    let mut grid = Grid::with_history(8, 2, 3);
    write(&mut grid, "aaaabbbb\nccccdddd\neeeeffff\ngg");
    grid.resize(4, 2);
    // The screen takes `ffff` and `gg`; three rows above them are kept.
    assert_eq!(history(&grid), ["cccc", "dddd", "eeee"]);
}

#[test]
fn the_scroll_region_goes_back_to_the_whole_screen() {
    let mut grid = grid(10, 3, "a\nb\nc");
    grid.set_region(0, 1);
    grid.resize(10, 4);
    grid.move_to(3, 0);
    grid.line_feed();
    assert_eq!(history(&grid), ["a"]);
}

#[test]
fn the_alternate_screen_is_only_cut() {
    let mut grid = Grid::new(10, 3);
    write(&mut grid, "abcd漢fgh\nx");
    grid.resize_cutting(5, 2);
    assert_eq!(grid.text(), "abcd\nx");
    assert_eq!(grid.history_len(), 0);
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

/// Lines of random lengths, without spaces so that nothing can be trimmed.
fn lines(numbers: &mut Numbers) -> Vec<String> {
    const CHARACTERS: [char; 4] = ['a', 'b', '漢', 'é'];
    (0..30)
        .map(|_| {
            (0..numbers.below(40))
                .map(|_| CHARACTERS[numbers.below(CHARACTERS.len())])
                .collect()
        })
        .collect()
}

#[test]
fn any_chain_of_widths_gives_the_same_text_back() {
    let mut numbers = Numbers(7);
    for round in 0..50 {
        let wanted = lines(&mut numbers).join("\n");
        let mut grid = Grid::with_history(13, 5, 10_000);
        write(&mut grid, &wanted);
        for _ in 0..8 {
            let columns = 2 + numbers.below(30) as u16;
            let rows = 1 + numbers.below(9) as u16;
            grid.resize(columns, rows);
        }
        grid.resize(100, 5);
        let got = grid.text_with_history();
        assert_eq!(
            got.trim_end_matches('\n'),
            wanted.trim_end_matches('\n'),
            "{round}"
        );
    }
}

#[test]
fn resizing_between_random_operations_breaks_nothing() {
    let mut numbers = Numbers(3);
    let mut grid = Grid::with_history(8, 4, 20);
    for step in 0..5_000 {
        match numbers.below(8) {
            0 => grid.resize(numbers.below(12) as u16, numbers.below(6) as u16),
            1 => grid.move_to(numbers.below(9), numbers.below(14)),
            2 => grid.line_feed(),
            3 => grid.delete_chars(numbers.below(4)),
            4 => grid.erase_in_line(Erase::ToEnd),
            _ => grid.print(['a', ' ', '漢', '\u{301}'][numbers.below(4)]),
        }
        let cursor = grid.cursor();
        assert!(
            cursor.row < grid.rows() && cursor.column < grid.columns(),
            "{step}"
        );
        assert!(
            grid.rows.iter().all(|r| r.cells.len() == grid.columns()),
            "{step}"
        );
        assert!(grid.history_len() <= 20, "{step}");
    }
}
