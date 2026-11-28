use super::*;
use crate::grid::{Cell, Erase};
use crate::terminal::Terminal;

fn damaged(grid: &mut Grid) -> Vec<usize> {
    let damage = grid.take_damage();
    let flags = damage.rows.iter().enumerate();
    flags.filter(|(_, &on)| on).map(|(row, _)| row).collect()
}

/// A grid of 10×4 whose damage has just been taken.
fn clean() -> Grid {
    let mut grid = Grid::new(10, 4);
    grid.take_damage();
    grid
}

#[test]
fn a_new_screen_is_all_damaged_once() {
    let mut grid = Grid::new(10, 3);
    assert_eq!(damaged(&mut grid), [0, 1, 2]);
    assert!(!grid.take_damage().any());
}

#[test]
fn writing_and_erasing_damage_their_row_and_moving_damages_nothing() {
    let mut grid = clean();
    grid.move_to(2, 3);
    grid.carriage_return();
    grid.tab();
    assert_eq!(damaged(&mut grid), [0; 0]);
    grid.print('a');
    assert_eq!(damaged(&mut grid), [2]);
    grid.print('\u{301}');
    assert_eq!(damaged(&mut grid), [2]);
    grid.erase_in_line(Erase::All);
    assert_eq!(damaged(&mut grid), [2]);
    grid.erase_in_display(Erase::ToEnd);
    assert_eq!(damaged(&mut grid), [2, 3]);
}

#[test]
fn scrolling_damages_the_rows_that_move() {
    let mut grid = clean();
    grid.set_region(1, 2);
    grid.scroll_up(1);
    assert_eq!(damaged(&mut grid), [1, 2]);
    grid.scroll_down(1);
    assert_eq!(damaged(&mut grid), [1, 2]);
}

#[test]
fn another_size_or_another_screen_damages_everything() {
    let mut terminal = Terminal::new(10, 3);
    let taken = |terminal: &mut Terminal| damaged(terminal.screen_mut());
    taken(&mut terminal);
    terminal.resize(10, 5);
    assert_eq!(taken(&mut terminal), [0, 1, 2, 3, 4]);
    terminal.enter_alternate_screen();
    assert_eq!(taken(&mut terminal), [0, 1, 2, 3, 4]);
    terminal.leave_alternate_screen();
    assert_eq!(taken(&mut terminal), [0, 1, 2, 3, 4]);
    terminal.reset();
    assert_eq!(taken(&mut terminal), [0, 1, 2, 3, 4]);
}

/// A fixed sequence of numbers that look random (xorshift).
struct Numbers(u64);

impl Numbers {
    fn below(&mut self, limit: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % limit as u64) as usize
    }
}

fn any_operation(terminal: &mut Terminal, numbers: &mut Numbers) -> usize {
    let operation = numbers.below(24);
    let (a, b) = (numbers.below(12), numbers.below(12));
    let character = ['a', 'b', ' ', '漢', '\u{301}'][numbers.below(5)];
    let erase = [Erase::ToEnd, Erase::ToStart, Erase::All][numbers.below(3)];
    let screen = terminal.screen_mut();
    match operation {
        0..=5 => screen.print(character),
        6 => screen.move_to(a, b),
        7 => screen.line_feed(),
        8 => screen.reverse_line_feed(),
        9 => screen.erase_in_line(erase),
        10 => screen.erase_in_display(erase),
        11 => screen.erase_chars(a),
        12 => screen.insert_chars(a),
        13 => screen.delete_chars(a),
        14 => screen.insert_lines(a),
        15 => screen.delete_lines(a),
        16 => screen.scroll_up(a),
        17 => screen.scroll_down(a),
        18 => screen.set_region(a, b),
        19 => screen.insert = a % 2 == 0,
        20 => terminal.enter_alternate_screen(),
        21 => terminal.leave_alternate_screen(),
        22 if b == 0 => terminal.resize(2 + a as u16, 2 + b as u16),
        23 if b == 0 => terminal.reset(),
        _ => screen.print('z'),
    }
    operation
}

fn cells(grid: &Grid) -> Vec<Vec<Cell>> {
    let row = |row| {
        let columns = 0..grid.columns();
        columns.filter_map(|c| grid.cell(row, c).cloned()).collect()
    };
    (0..grid.rows()).map(row).collect()
}

#[test]
fn after_any_operation_every_row_that_changed_is_damaged() {
    let mut terminal = Terminal::new(8, 5);
    let mut numbers = Numbers(0x9E37_79B9_7F4A_7C15);
    terminal.screen_mut().take_damage();
    let mut drawn = cells(terminal.screen());
    for step in 0..20_000 {
        let operation = any_operation(&mut terminal, &mut numbers);
        let damage = terminal.screen_mut().take_damage().rows;
        let now = cells(terminal.screen());
        assert_eq!(damage.len(), now.len(), "step {step}, {operation}");
        for (row, found) in now.iter().enumerate() {
            let changed = drawn.get(row) != Some(found);
            assert!(
                !changed || damage[row],
                "step {step}, {operation}, row {row}"
            );
        }
        drawn = now;
    }
}
