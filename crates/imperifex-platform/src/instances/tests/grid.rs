//! The grid of a terminal, with the square letters of these tests.

use imperifex_domain::font::CellSize;
use imperifex_domain::grid::{Attributes, Color, Damage, Style};
use imperifex_domain::interpret::apply;
use imperifex_domain::picture::TerminalPicture;
use imperifex_domain::sequence::Sequence;
use imperifex_domain::terminal::Terminal;
use imperifex_domain::theme::{palette, Palette, Theme};

use super::*;

const CELL: CellSize = CellSize {
    width: 10,
    height: 20,
    baseline: 15,
};

/// Cells of 10 by 20 in an area at (100, 50) with room for 6 by 3.
struct Window {
    terminal: Terminal,
    area: Rect,
    palette: &'static Palette,
    focused: bool,
    batch: Batch,
    atlas: Atlas,
}

impl Window {
    /// With the cursor hidden, to look only at the cells.
    fn new() -> Self {
        let mut terminal = Terminal::new(6, 3);
        terminal.modes.cursor_visible = false;
        Self {
            terminal,
            area: Rect {
                x: 100,
                y: 50,
                width: 60,
                height: 60,
            },
            palette: palette(Theme::Dark),
            focused: true,
            batch: Batch::default(),
            atlas: Atlas::new(256),
        }
    }

    fn write(&mut self, text: &str) {
        for character in text.chars() {
            apply(&mut self.terminal, Sequence::Print(character));
        }
    }

    fn pen(&mut self, style: Style) {
        self.terminal.screen_mut().pen = style;
    }

    /// A frame with this damage.
    fn frame_with(&mut self, damage: &Damage) -> Vec<Instance> {
        let picture = TerminalPicture {
            terminal: &self.terminal,
            damage,
            area: self.area,
            palette: self.palette,
            adjust_contrast: true,
            cell: CELL,
            letters: 16.0,
            line: 1,
            focused: self.focused,
        };
        let source = SquareLetters::default();
        let list = DrawList::default();
        self.batch
            .fill(&list, Some(&picture), &source, &mut self.atlas);
        self.batch.instances.clone()
    }

    /// A frame with what changed since the last one.
    fn frame(&mut self) -> Vec<Instance> {
        let damage = self.terminal.screen_mut().take_damage();
        self.frame_with(&damage)
    }
}

/// Nothing changed in any of the three rows.
fn untouched() -> Damage {
    Damage {
        rows: vec![false; 3],
    }
}

fn glyphs(instances: &[Instance]) -> Vec<[f32; 4]> {
    let all = instances.iter().filter(|one| one.glyph > 0.5);
    all.map(|one| one.area).collect()
}

fn solids(instances: &[Instance]) -> Vec<([f32; 4], [f32; 4])> {
    let all = instances.iter().filter(|one| one.glyph < 0.5);
    all.map(|one| (one.area, one.color)).collect()
}

fn colour(rgb: Rgb) -> [f32; 4] {
    solid(Rect::default(), rgb).color
}

fn on_red() -> Style {
    Style {
        background: Color::Rgb(200, 0, 0),
        ..Style::default()
    }
}

#[test]
fn each_letter_stands_on_the_baseline_of_its_cell_and_spaces_add_nothing() {
    let mut window = Window::new();
    window.write("a b");
    // A square letter is one right of the pen and five above the baseline.
    assert_eq!(
        glyphs(&window.frame()),
        [[101.0, 60.0, 4.0, 6.0], [121.0, 60.0, 4.0, 6.0]]
    );
}

#[test]
fn only_a_background_unlike_the_outputs_is_painted_and_all_go_under_the_letters() {
    let mut window = Window::new();
    window.write("a");
    window.terminal.screen_mut().move_to(1, 0);
    window.pen(on_red());
    window.write("b");
    let found = window.frame();
    let red = colour(Rgb(200, 0, 0));
    assert_eq!(solids(&found), [([100.0, 70.0, 10.0, 20.0], red)]);
    // The background of the second row comes before the letter of the first.
    assert!(found[0].glyph < 0.5);
}

#[test]
fn a_row_without_damage_is_not_made_again_and_one_with_it_is() {
    let mut window = Window::new();
    window.write("a");
    let first = window.frame();
    window.write("b");
    assert_eq!(window.frame_with(&untouched()), first);
    assert_eq!(glyphs(&window.frame()).len(), 2);
}

#[test]
fn another_palette_or_an_emptied_atlas_makes_every_row_again() {
    let mut window = Window::new();
    window.write("a");
    window.frame();
    window.write("b");
    window.palette = palette(Theme::Light);
    let light = window.frame_with(&untouched());
    assert_eq!(glyphs(&light).len(), 2);
    window.write("c");
    window.atlas.clear();
    assert_eq!(glyphs(&window.frame_with(&untouched())).len(), 3);
    assert!(!window.batch.uploads.is_empty());
}

#[test]
fn only_whole_cells_that_fit_in_the_area_are_drawn() {
    let mut window = Window::new();
    window.write("abcdef");
    window.terminal.screen_mut().move_to(2, 0);
    window.write("z");
    window.area.width = 39;
    window.area.height = 59;
    let found = glyphs(&window.frame());
    assert_eq!(found.len(), 3, "{found:?}");
}

#[test]
fn the_focused_cursor_is_a_block_with_its_letter_in_the_colour_of_the_output() {
    let mut window = Window::new();
    window.write("a");
    window.terminal.screen_mut().carriage_return();
    window.terminal.modes.cursor_visible = true;
    let found = window.frame();
    let accent = colour(window.palette.accent);
    assert_eq!(solids(&found), [([100.0, 50.0, 10.0, 20.0], accent)]);
    let over = found.last().expect("the letter over the cursor");
    assert!(over.glyph > 0.5);
    assert_eq!(over.color, colour(window.palette.surface));
}

#[test]
fn without_focus_the_cursor_is_only_its_outline_and_hidden_it_is_nothing() {
    let mut window = Window::new();
    window.terminal.modes.cursor_visible = true;
    window.focused = false;
    let sides: Vec<_> = solids(&window.frame())
        .iter()
        .map(|&(area, _)| area)
        .collect();
    assert_eq!(
        sides,
        [
            [100.0, 50.0, 10.0, 1.0],
            [100.0, 69.0, 10.0, 1.0],
            [100.0, 50.0, 1.0, 20.0],
            [109.0, 50.0, 1.0, 20.0],
        ]
    );
    window.terminal.modes.cursor_visible = false;
    assert!(window.frame().is_empty());
}

#[test]
fn on_either_half_of_a_wide_character_the_cursor_covers_both() {
    let mut window = Window::new();
    window.write("漢");
    window.terminal.modes.cursor_visible = true;
    for column in [0, 1] {
        window.terminal.screen_mut().move_to(0, column);
        let found = window.frame();
        let blocks: Vec<_> = solids(&found).iter().map(|&(area, _)| area).collect();
        assert_eq!(blocks, [[100.0, 50.0, 20.0, 20.0]], "column {column}");
    }
}

#[test]
fn bold_and_italic_are_asked_of_their_faces() {
    let mut window = Window::new();
    for attribute in [Attributes::BOLD, Attributes::ITALIC] {
        window.pen(Style {
            attributes: attribute,
            ..Style::default()
        });
        window.write("a");
    }
    window.frame();
    // The same letter twice, in two faces: two pictures.
    assert_eq!(window.batch.uploads.len(), 2);
}

#[test]
fn underline_goes_below_the_baseline_and_strike_through_the_letters() {
    let mut window = Window::new();
    window.pen(Style {
        attributes: Attributes::UNDERLINE.with(Attributes::STRIKE),
        ..Style::default()
    });
    window.write("a");
    let tops: Vec<_> = solids(&window.frame())
        .iter()
        .map(|&(area, _)| (area[1], area[3]))
        .collect();
    assert_eq!(tops, [(66.0, 1.0), (61.0, 1.0)]);
}

#[test]
fn a_mark_is_drawn_over_its_letter_whichever_side_of_the_pen_it_is_made_for() {
    let mut window = Window::new();
    // The first is drawn ahead of the pen, the second behind it; neither moves
    // it, and the joiner is not drawn.
    window.write("e\u{301}\u{200D}e\u{20D7}");
    assert_eq!(
        glyphs(&window.frame()),
        [
            [101.0, 60.0, 4.0, 6.0],
            [101.0, 60.0, 4.0, 6.0],
            [111.0, 60.0, 4.0, 6.0],
            [115.0, 60.0, 4.0, 6.0],
        ]
    );
}

#[test]
fn what_floats_is_drawn_over_the_grid() {
    let mut window = Window::new();
    window.write("a");
    let damage = window.terminal.screen_mut().take_damage();
    let picture = TerminalPicture {
        terminal: &window.terminal,
        damage: &damage,
        area: window.area,
        palette: window.palette,
        adjust_contrast: true,
        cell: CELL,
        letters: 16.0,
        line: 1,
        focused: true,
    };
    let list = DrawList {
        floating_rects: vec![(window.area, Rgb::WHITE)],
        ..DrawList::default()
    };
    let source = SquareLetters::default();
    window
        .batch
        .fill(&list, Some(&picture), &source, &mut window.atlas);
    let last = window.batch.instances.last().expect("something");
    assert_eq!(last.color, colour(Rgb::WHITE));
    assert!(last.glyph < 0.5);
}
