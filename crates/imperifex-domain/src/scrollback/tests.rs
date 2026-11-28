use super::*;
use crate::grid::Cell;
use crate::interpret::apply;
use crate::sequence::Sequence;
use crate::terminal::Terminal;

/// A screen of 10 by 3 with a history of 5 rows at most.
fn terminal() -> Terminal {
    Terminal::with_history(10, 3, 5)
}

/// Writes one letter per line (0 is `a`), each leaving the cursor on the next.
fn lines(terminal: &mut Terminal, numbers: std::ops::Range<u8>) {
    for number in numbers {
        let letter = char::from(b'a' + number);
        apply(terminal, Sequence::Print(letter));
        for &control in b"\r\n" {
            apply(terminal, Sequence::Control(control));
        }
    }
}

/// The first character of each row seen.
fn seen(terminal: &Terminal, view: View) -> String {
    let screen = terminal.screen();
    (0..screen.rows())
        .map(|row| {
            let first = -isize::try_from(view.back()).unwrap_or(0);
            let cells = screen.seen_row(first, row);
            cells.first().map_or(' ', Cell::base)
        })
        .collect()
}

#[test]
fn the_view_goes_back_as_far_as_the_history_and_no_further_than_the_screen() {
    let mut terminal = terminal();
    lines(&mut terminal, 0..6);
    let mut view = View::default();
    assert_eq!(seen(&terminal, view), "ef ");
    view.scroll(3, terminal.screen());
    assert_eq!(seen(&terminal, view), "bcd");
    view.scroll(100, terminal.screen());
    assert_eq!((view.back(), seen(&terminal, view)), (4, "abc".into()));
    view.scroll(-100, terminal.screen());
    assert_eq!((view.back(), seen(&terminal, view)), (0, "ef ".into()));
}

#[test]
fn a_view_that_is_back_stays_on_its_text_while_output_arrives() {
    let mut terminal = terminal();
    lines(&mut terminal, 0..5);
    let mut view = View::default();
    view.scroll(2, terminal.screen());
    assert_eq!(seen(&terminal, view), "bcd");
    lines(&mut terminal, 5..7);
    view.follow(terminal.screen());
    assert_eq!(seen(&terminal, view), "bcd");
    // The history is full: the oldest row it showed is forgotten.
    lines(&mut terminal, 7..9);
    view.follow(terminal.screen());
    assert_eq!((view.back(), seen(&terminal, view)), (5, "cde".into()));
}

#[test]
fn a_view_on_the_screen_stays_there() {
    let mut terminal = terminal();
    let mut view = View::default();
    lines(&mut terminal, 0..6);
    view.follow(terminal.screen());
    assert_eq!(view.back(), 0);
    view.scroll(2, terminal.screen());
    view.to_screen();
    lines(&mut terminal, 6..8);
    view.follow(terminal.screen());
    assert_eq!(view.back(), 0);
}

#[test]
fn clearing_the_history_or_the_alternate_screen_leaves_nothing_to_look_back_at() {
    let mut terminal = terminal();
    lines(&mut terminal, 0..6);
    let mut view = View::default();
    view.scroll(2, terminal.screen());
    terminal.screen_mut().clear_history();
    view.follow(terminal.screen());
    assert_eq!(view.back(), 0);
    lines(&mut terminal, 0..6);
    view.scroll(2, terminal.screen());
    terminal.enter_alternate_screen();
    view.follow(terminal.screen());
    assert_eq!(view.back(), 0);
}

fn modes(mouse_reports: bool, mouse_digits: bool) -> Modes {
    Modes {
        mouse_reports,
        mouse_digits,
        ..Modes::default()
    }
}

/// A turn on the main screen of a program that did not ask for the mouse.
fn plain(wheel: &mut Wheel, notches: f32) -> Turned {
    wheel.turn(notches, Modes::default(), false, (0, 0))
}

#[test]
fn on_the_main_screen_a_notch_is_three_rows() {
    let mut wheel = Wheel::default();
    assert_eq!(plain(&mut wheel, 1.0), Turned::Scroll(3));
    assert_eq!(plain(&mut wheel, -2.0), Turned::Scroll(-6));
}

#[test]
fn a_touchpads_small_turns_add_up_and_turning_back_drops_the_rest() {
    let mut wheel = Wheel::default();
    // Three quarters of a row, and then a row and a half.
    assert_eq!(plain(&mut wheel, 0.25), Turned::Nothing);
    assert_eq!(plain(&mut wheel, 0.25), Turned::Scroll(1));
    // Half a row was left upwards: it does not eat into the way down.
    assert_eq!(plain(&mut wheel, -0.375), Turned::Scroll(-1));
}

#[test]
fn a_program_that_asked_for_the_mouse_gets_one_report_a_notch_where_it_is() {
    let mut wheel = Wheel::default();
    // In the alternate screen or not, as Claude Code in full screen asks.
    for alternate in [true, false] {
        let up = wheel.turn(1.0, modes(true, true), alternate, (9, 4));
        assert_eq!(up, Turned::Send(b"\x1b[<64;10;5M".to_vec()));
    }
    let down = wheel.turn(-2.0, modes(true, true), true, (0, 0));
    let twice = b"\x1b[<65;1;1M\x1b[<65;1;1M".to_vec();
    assert_eq!(down, Turned::Send(twice));
}

#[test]
fn without_digits_the_report_is_a_byte_each_and_a_far_place_is_cut() {
    let found = Wheel::default().turn(1.0, modes(true, false), true, (9, 500));
    assert_eq!(found, Turned::Send(vec![0x1B, b'[', b'M', 96, 42, 255]));
}

#[test]
fn the_alternate_screen_without_the_mouse_gets_arrows() {
    let mut wheel = Wheel::default();
    let found = wheel.turn(1.0, Modes::default(), true, (0, 0));
    assert_eq!(found, Turned::Send(b"\x1b[A".repeat(3)));
    let application = Modes {
        application_cursor_keys: true,
        ..Modes::default()
    };
    let found = wheel.turn(-1.0, application, true, (0, 0));
    assert_eq!(found, Turned::Send(b"\x1bOB".repeat(3)));
}
