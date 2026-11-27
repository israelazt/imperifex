use super::*;
use crate::grid::Position;

fn at(row: usize, column: usize) -> Position {
    Position { row, column }
}

fn send_csi(grid: &mut Grid, params: &[u16], action: char) {
    apply(
        grid,
        Sequence::Csi {
            params,
            intermediates: &[],
            action,
        },
    );
}

fn print(grid: &mut Grid, text: &str) {
    for character in text.chars() {
        apply(grid, Sequence::Print(character));
    }
}

/// A 10 by 5 grid with the cursor at row 2, column 4.
fn centred() -> Grid {
    let mut grid = Grid::new(10, 5);
    grid.move_to(2, 4);
    grid
}

#[test]
fn each_movement_goes_where_it_says() {
    let cases: [(char, &[u16], (usize, usize)); 11] = [
        ('A', &[2], (0, 4)),
        ('B', &[2], (4, 4)),
        ('C', &[3], (2, 7)),
        ('D', &[3], (2, 1)),
        ('E', &[1], (3, 0)),
        ('F', &[1], (1, 0)),
        ('G', &[7], (2, 6)),
        ('d', &[5], (4, 4)),
        ('H', &[2, 3], (1, 2)),
        ('f', &[4, 9], (3, 8)),
        ('H', &[0], (0, 0)),
    ];
    for (action, params, (row, column)) in cases {
        let mut grid = centred();
        send_csi(&mut grid, params, action);
        assert_eq!(grid.cursor(), at(row, column), "{action} {params:?}");
    }
}

#[test]
fn nothing_and_0_move_one_step() {
    for params in [&[][..], &[0][..]] {
        let mut grid = centred();
        send_csi(&mut grid, params, 'C');
        assert_eq!(grid.cursor(), at(2, 5), "{params:?}");
    }
}

#[test]
fn a_huge_movement_stays_inside_the_grid() {
    for action in ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'd', 'H'] {
        let mut grid = centred();
        send_csi(&mut grid, &[u16::MAX, u16::MAX], action);
        let cursor = grid.cursor();
        assert!(cursor.row < 5 && cursor.column < 10, "{action}");
    }
}

#[test]
fn the_controls_move_the_cursor() {
    let mut grid = Grid::new(20, 5);
    print(&mut grid, "ab");
    let moves = [
        (0x08, at(0, 1)),
        (0x09, at(0, 8)),
        (0x0A, at(1, 8)),
        (0x0B, at(2, 8)),
        (0x0C, at(3, 8)),
        (0x0D, at(3, 0)),
        // The bell does nothing to the grid.
        (0x07, at(3, 0)),
    ];
    for (byte, expected) in moves {
        apply(&mut grid, Sequence::Control(byte));
        assert_eq!(grid.cursor(), expected, "{byte:#x}");
    }
}

#[test]
fn erasing_follows_its_parameter() {
    for (action, param, expected) in [
        ('K', 0, "aaaa\nbb\ncccc"),
        ('K', 1, "aaaa\n   b\ncccc"),
        ('K', 2, "aaaa\n\ncccc"),
        ('J', 0, "aaaa\nbb\n"),
        ('J', 1, "\n   b\ncccc"),
        ('J', 2, "\n\n"),
        // The history is not part of the visible screen.
        ('J', 3, "aaaa\nbbbb\ncccc"),
    ] {
        let mut grid = Grid::new(10, 3);
        for (row, text) in ["aaaa", "bbbb", "cccc"].iter().enumerate() {
            grid.move_to(row, 0);
            print(&mut grid, text);
        }
        grid.move_to(1, 2);
        send_csi(&mut grid, &[param], action);
        assert_eq!(grid.text(), expected, "{action} {param}");
    }
}

#[test]
fn x_empties_that_many_cells_without_moving() {
    let mut grid = Grid::new(10, 1);
    print(&mut grid, "abcdef");
    grid.move_to(0, 1);
    send_csi(&mut grid, &[3], 'X');
    assert_eq!(grid.text(), "a   ef");
    send_csi(&mut grid, &[u16::MAX], 'X');
    assert_eq!(grid.text(), "a");
    assert_eq!(grid.cursor(), at(0, 1));
}

#[test]
fn esc_d_goes_down_and_esc_e_starts_a_new_line() {
    let mut grid = centred();
    let esc = |action| Sequence::Esc {
        intermediates: &[],
        action,
    };
    apply(&mut grid, esc(b'D'));
    assert_eq!(grid.cursor(), at(3, 4));
    apply(&mut grid, esc(b'E'));
    assert_eq!(grid.cursor(), at(4, 0));
}

#[test]
fn a_marker_changes_the_meaning_so_the_sequence_is_ignored() {
    let mut grid = centred();
    apply(
        &mut grid,
        Sequence::Csi {
            params: &[2],
            intermediates: b"?",
            action: 'J',
        },
    );
    apply(
        &mut grid,
        Sequence::Esc {
            intermediates: b"(",
            action: b'E',
        },
    );
    assert_eq!(grid.cursor(), at(2, 4));
}

#[test]
fn what_is_not_known_changes_nothing() {
    let mut grid = centred();
    print(&mut grid, "x");
    let before = (grid.text(), grid.cursor());
    send_csi(&mut grid, &[31], 'm');
    send_csi(&mut grid, &[], 'z');
    apply(&mut grid, Sequence::Osc(&[b"0", b"titulo"]));
    assert_eq!((grid.text(), grid.cursor()), before);
}
