//! The grid of a terminal as instances. Those of each row are kept, and made
//! again only for a row that changed.

use imperifex_domain::atlas::GlyphKey;
use imperifex_domain::draw::Rect;
use imperifex_domain::font::CellSize;
use imperifex_domain::grid::{Attributes, Cell, Grid, Span};
use imperifex_domain::picture::TerminalPicture;
use imperifex_domain::theme::{cell_colors, Palette, Rgb};

use super::{solid, Full, GridLook, Instance, Letters};

/// What the instances of every row depend on, besides what the row says. With
/// any of it changed they are all made again.
#[derive(Debug, Clone, Copy, PartialEq)]
struct MadeFor {
    area: Rect,
    palette: &'static Palette,
    adjust_contrast: bool,
    cell: CellSize,
    letters: f32,
    line: u32,
    columns: usize,
    rows: usize,
    back: usize,
    /// Of the atlas, where the glyphs of the rows are.
    atlas: u64,
}

#[derive(Debug, Default)]
struct Row {
    /// All of them go under every glyph and line: a letter may reach into the
    /// row below.
    backgrounds: Vec<Instance>,
    fronts: Vec<Instance>,
}

#[derive(Debug, Default)]
pub struct Rows {
    made_for: Option<MadeFor>,
    rows: Vec<Row>,
}

/// How many columns and rows are seen.
#[derive(Clone, Copy)]
struct Seen {
    columns: usize,
    rows: usize,
}

/// Only the cells that fit whole in the area: while the window changes size,
/// the grid may be larger than its room.
fn seen(picture: &TerminalPicture<'_>) -> Seen {
    let screen = picture.terminal.screen();
    let fit = |room: u32, side: u32| usize::try_from(room / side.max(1)).unwrap_or(0);
    Seen {
        columns: screen
            .columns()
            .min(fit(picture.area.width, picture.cell.width)),
        rows: screen
            .rows()
            .min(fit(picture.area.height, picture.cell.height)),
    }
}

fn offset(count: usize, side: u32) -> i32 {
    let count = i32::try_from(count).unwrap_or(i32::MAX);
    count.saturating_mul(i32::try_from(side).unwrap_or(i32::MAX))
}

/// Where `columns` cells from the one at `row` and `column` are.
fn place(picture: &TerminalPicture<'_>, row: usize, column: usize, columns: u32) -> Rect {
    Rect {
        x: picture
            .area
            .x
            .saturating_add(offset(column, picture.cell.width)),
        y: picture
            .area
            .y
            .saturating_add(offset(row, picture.cell.height)),
        width: picture.cell.width.saturating_mul(columns),
        height: picture.cell.height,
    }
}

fn width_in_cells(cell: &Cell) -> u32 {
    if cell.span() == Span::Double {
        2
    } else {
        1
    }
}

/// Characters that only say how others join or look: with no shaping of text
/// they would be drawn as missing ones.
fn is_unseen(character: char) -> bool {
    matches!(
        character,
        '\u{200B}'..='\u{200F}' | '\u{2060}' | '\u{FE00}'..='\u{FE0F}' | '\u{E0000}'..='\u{E007F}'
    )
}

/// Whether the glyph of a mark is drawn to the left of the pen: such a one
/// goes where the pen is after its letter. Others, as those of Cascadia Mono,
/// are drawn from where the letter's pen was; how far a mark moves it does not
/// tell them apart.
fn hangs_back(letters: &mut Letters<'_>, mark: GlyphKey) -> Result<bool, Full> {
    let slot = letters.slot(mark)?;
    let twice_the_middle = i64::from(slot.left) * 2 + i64::from(slot.width);
    Ok(twice_the_middle <= 0)
}

/// The glyphs of a cell whose columns are `area`, in `ink`.
fn write_cell(
    picture: &TerminalPicture<'_>,
    cell: &Cell,
    area: Rect,
    ink: Rgb,
    letters: &mut Letters<'_>,
    into: &mut Vec<Instance>,
) -> Result<(), Full> {
    let has = |attribute| cell.style().attributes.has(attribute);
    let look = GridLook {
        bold: has(Attributes::BOLD),
        italic: has(Attributes::ITALIC),
    };
    let baseline = area.y.saturating_add(offset(1, picture.cell.baseline));
    let mut result = Ok(());
    let characters = std::iter::once(cell.base()).chain(cell.marks().chars());
    for (at, character) in characters.enumerate() {
        if character == ' ' || is_unseen(character) {
            continue;
        }
        let shaped = letters.source.shape_grid(character, look, picture.letters);
        let Some(shaped) = shaped else { continue };
        let after = at > 0 && hangs_back(letters, shaped.key)?;
        let pen = if after {
            area.x.saturating_add(offset(1, area.width))
        } else {
            area.x
        };
        result = letters
            .glyph(shaped.key, (pen, baseline), ink, into)
            .and(result);
    }
    result
}

/// The lines under and through the text of a cell.
fn lines(
    picture: &TerminalPicture<'_>,
    cell: &Cell,
    area: Rect,
    ink: Rgb,
    into: &mut Vec<Instance>,
) {
    let CellSize {
        height, baseline, ..
    } = picture.cell;
    let thick = picture.line.max(1);
    let mut line = |from_top: u32| {
        let top = from_top.min(height.saturating_sub(thick));
        let strip = Rect {
            y: area.y.saturating_add(offset(1, top)),
            height: thick,
            ..area
        };
        into.push(solid(strip, ink));
    };
    let attributes = cell.style().attributes;
    if attributes.has(Attributes::UNDERLINE) {
        line(baseline.saturating_add(thick));
    }
    if attributes.has(Attributes::STRIKE) {
        // About the middle of the small letters.
        line(baseline - baseline * 3 / 10);
    }
}

fn make_row(
    picture: &TerminalPicture<'_>,
    row: usize,
    columns: usize,
    letters: &mut Letters<'_>,
    into: &mut Row,
) -> Result<(), Full> {
    into.backgrounds.clear();
    into.fronts.clear();
    let cells = picture.terminal.screen().seen_row(picture.back, row);
    let mut result = Ok(());
    for column in 0..columns {
        let Some(cell) = cells.get(column) else {
            break;
        };
        let (ink, back) = cell_colors(cell.style(), picture.palette, picture.adjust_contrast);
        if back != picture.palette.surface {
            into.backgrounds
                .push(solid(place(picture, row, column, 1), back));
        }
        // The second column of a wide character is drawn with the first, and
        // half of one is not drawn.
        let wide = width_in_cells(cell);
        let cut = column + usize::try_from(wide).unwrap_or(1) > columns;
        if cell.span() == Span::Continuation || cut {
            continue;
        }
        let area = place(picture, row, column, wide);
        result = write_cell(picture, cell, area, ink, letters, &mut into.fronts).and(result);
        lines(picture, cell, area, ink, &mut into.fronts);
    }
    result
}

/// The cell of the screen the cursor is on: on the second column of a wide
/// character, that of its first.
fn under_cursor(screen: &Grid) -> Option<(usize, usize, &Cell)> {
    let at = screen.cursor();
    let on = screen.cell(at.row, at.column)?;
    let column = if on.span() == Span::Continuation {
        at.column.saturating_sub(1)
    } else {
        at.column
    };
    screen
        .cell(at.row, column)
        .map(|cell| (at.row, column, cell))
}

/// A block with its character over it, or only its outline when the window
/// does not have the keyboard. It does not blink (T-D6).
fn cursor(
    picture: &TerminalPicture<'_>,
    seen: Seen,
    letters: &mut Letters<'_>,
    into: &mut Vec<Instance>,
) -> Result<(), Full> {
    if !picture.terminal.modes.cursor_visible {
        return Ok(());
    }
    let Some((row, column, cell)) = under_cursor(picture.terminal.screen()) else {
        return Ok(());
    };
    // Looking back, the screen's rows are that much lower, or out of sight.
    let row = row.saturating_add(picture.back);
    if row >= seen.rows || column >= seen.columns {
        return Ok(());
    }
    let area = place(picture, row, column, width_in_cells(cell));
    let accent = picture.palette.accent;
    if picture.focused {
        into.push(solid(area, accent));
        let ink = picture.palette.surface;
        return write_cell(picture, cell, area, ink, letters, into);
    }
    into.extend(outline(area, picture.line.max(1)).map(|side| solid(side, accent)));
    Ok(())
}

/// The four sides of `area`, `thick` pixels each.
fn outline(area: Rect, thick: u32) -> impl Iterator<Item = Rect> {
    let right = area
        .x
        .saturating_add(offset(1, area.width.saturating_sub(thick)));
    let bottom = area
        .y
        .saturating_add(offset(1, area.height.saturating_sub(thick)));
    let across = |y| Rect {
        y,
        height: thick,
        ..area
    };
    let down = |x| Rect {
        x,
        width: thick,
        ..area
    };
    [across(area.y), across(bottom), down(area.x), down(right)].into_iter()
}

impl Rows {
    /// Makes every row again the next time.
    pub fn forget(&mut self) {
        self.made_for = None;
    }

    /// Adds the grid and its cursor to `into`.
    pub fn draw(
        &mut self,
        picture: &TerminalPicture<'_>,
        letters: &mut Letters<'_>,
        into: &mut Vec<Instance>,
    ) -> Result<(), Full> {
        let seen = seen(picture);
        let wanted = MadeFor {
            area: picture.area,
            palette: picture.palette,
            adjust_contrast: picture.adjust_contrast,
            cell: picture.cell,
            letters: picture.letters,
            line: picture.line,
            columns: seen.columns,
            rows: seen.rows,
            back: picture.back,
            atlas: letters.atlas.generation(),
        };
        // Looking back, what each row shows moves with the history, which the
        // damage of the screen does not tell.
        let all = self.made_for != Some(wanted) || picture.back > 0;
        self.made_for = Some(wanted);
        self.rows.resize_with(seen.rows, Row::default);
        let mut result = Ok(());
        for (at, row) in self.rows.iter_mut().enumerate() {
            // A row the damage says nothing of is taken as changed.
            if all || picture.damage.rows.get(at) != Some(&false) {
                result = make_row(picture, at, seen.columns, letters, row).and(result);
            }
        }
        into.extend(self.rows.iter().flat_map(|row| &row.backgrounds));
        into.extend(self.rows.iter().flat_map(|row| &row.fronts));
        cursor(picture, seen, letters, into).and(result)
    }
}
