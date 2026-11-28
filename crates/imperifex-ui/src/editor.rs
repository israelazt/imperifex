//! The command editor under the output (T-D22): a box with the shell's prompt
//! and the command, and under it what its keys do.

use imperifex_domain::draw::{DrawList, Rect, Rounded, UiFont};
use imperifex_domain::ports::TextMeasure;
use imperifex_domain::space::Space;
use imperifex_domain::view::{EditorView, Model};

use crate::metrics;
use crate::text::Ink;

mod hints;
mod wrap;

fn signed(pixels: u32) -> i32 {
    i32::try_from(pixels).unwrap_or(i32::MAX)
}

fn count(rows: usize) -> u32 {
    u32::try_from(rows).unwrap_or(u32::MAX)
}

/// The sizes of what is drawn, in physical pixels.
struct Sizes {
    /// The line around a box, and between the output and the editor.
    border: u32,
    /// A row of the prompt or the command.
    line: u32,
    /// The line of the keys.
    hints: u32,
    /// How many characters fit in a row of the box.
    per_row: usize,
}

fn sizes(model: &Model, view: &EditorView, measure: &dyn TextMeasure) -> Sizes {
    let scale = model.scale;
    // The box has the base space at its sides, and inside it (`M`) more.
    let taken = (Space::L.px(scale) + Space::M.px(scale)) * 2;
    let room = model.window.0.saturating_sub(taken);
    // Every character of a font of the output is as wide as this one.
    let one = measure.width(&'0'.to_string(), UiFont::Mono, view.letters);
    let secondary = metrics::px(metrics::SECONDARY_TEXT, scale);
    Sizes {
        border: metrics::px(metrics::BORDER, scale).max(1),
        line: measure.line_height(UiFont::Mono, view.letters),
        hints: measure.line_height(UiFont::Text, secondary),
        per_row: usize::try_from(room / one.max(1)).unwrap_or(1).max(1),
    }
}

/// Where the parts of the editor are, against `bottom`.
struct Layout {
    sizes: Sizes,
    prompt: Vec<String>,
    command: wrap::Command,
    top: i32,
    /// The box.
    area: Rect,
    /// Where the first row of the command is.
    left: i32,
    first_row: i32,
}

fn layout(model: &Model, view: &EditorView, bottom: i32, measure: &dyn TextMeasure) -> Layout {
    let sizes = sizes(model, view, measure);
    let command = wrap::command(view, sizes.per_row);
    let prompt = wrap::all(&view.prompt, sizes.per_row);
    let scale = model.scale;
    let (small, middle) = (Space::S.px(scale), Space::M.px(scale));
    let rows = count(prompt.len() + command.rows.len());
    let inside = middle * 2 + rows * sizes.line + small;
    let total = sizes.border + middle + inside + small + sizes.hints + middle;
    let top = bottom.saturating_sub(signed(total));
    let side = Space::L.px(scale);
    let area = Rect {
        x: signed(side),
        y: top.saturating_add(signed(sizes.border + middle)),
        width: model.window.0.saturating_sub(side * 2),
        height: inside,
    };
    let above = middle + count(prompt.len()) * sizes.line + small;
    Layout {
        left: area.x.saturating_add(signed(middle)),
        first_row: area.y.saturating_add(signed(above)),
        sizes,
        prompt,
        command,
        top,
        area,
    }
}

/// The place of the text under the point `at`, if it is in the box: its line
/// and how many characters of it are before, the closest there is.
pub fn place_at(
    model: &Model,
    view: &EditorView,
    bottom: i32,
    (x, y): (i32, i32),
    measure: &dyn TextMeasure,
) -> Option<(usize, usize)> {
    let layout = layout(model, view, bottom, measure);
    let end = layout.area.y.saturating_add(signed(layout.area.height));
    if y < layout.area.y || y >= end || x < layout.area.x {
        return None;
    }
    let line = layout.sizes.line.max(1);
    let down = u32::try_from(y.saturating_sub(layout.first_row)).unwrap_or(0) / line;
    let last = layout.command.rows.len().checked_sub(1)?;
    let row = usize::try_from(down).unwrap_or(last).min(last);
    let text = &layout.command.rows[row];
    let one = measure.width(&'0'.to_string(), UiFont::Mono, view.letters);
    // Past the middle of a letter is after it.
    let along = u32::try_from(x.saturating_sub(layout.left)).unwrap_or(0) + one / 2;
    let column = usize::try_from(along / one.max(1)).unwrap_or(0);
    let (of_line, before) = *layout.command.starts.get(row)?;
    Some((of_line, before + column.min(text.chars().count())))
}

/// Draws the editor against `bottom`, as tall as what it holds, and says where
/// its top is: the output ends over it.
pub fn draw(
    model: &Model,
    view: &EditorView,
    bottom: i32,
    measure: &dyn TextMeasure,
    into: &mut DrawList,
) -> i32 {
    let Layout {
        sizes,
        prompt,
        command,
        top,
        area,
        left,
        first_row,
    } = layout(model, view, bottom, measure);
    let (small, middle) = (Space::S.px(model.scale), Space::M.px(model.scale));
    let separator = Rect {
        x: 0,
        y: top,
        width: model.window.0,
        height: sizes.border,
    };
    into.rects.push((separator, model.palette.line));
    frame(model, area, sizes.border, into);

    let ink = |color| Ink {
        font: UiFont::Mono,
        size: view.letters,
        color,
    };
    let mut y = area.y.saturating_add(signed(middle));
    for row in prompt {
        into.texts
            .push(ink(model.palette.accent).run((left, y), row));
        y = y.saturating_add(signed(sizes.line));
    }
    let written = Written {
        left,
        top: first_row,
        line: sizes.line,
    };
    written.draw(model, view, &command, measure, into);

    let under = area.y.saturating_add(signed(area.height + small));
    hints::draw(model, view, (area.x, under), measure, into);
    top
}

/// The box: raised, with a line around it that shows it has the keyboard.
fn frame(model: &Model, area: Rect, border: u32, into: &mut DrawList) {
    let radius = metrics::px(metrics::EDITOR_RADIUS, model.scale);
    let around = if model.focused {
        model.palette.accent
    } else {
        model.palette.line
    };
    into.rounded.push(Rounded {
        area,
        radius,
        color: around,
    });
    into.rounded.push(Rounded {
        area: Rect {
            x: area.x.saturating_add(signed(border)),
            y: area.y.saturating_add(signed(border)),
            width: area.width.saturating_sub(border * 2),
            height: area.height.saturating_sub(border * 2),
        },
        radius: radius.saturating_sub(border),
        color: model.palette.raised,
    });
}

/// Where the rows of the command are.
struct Written {
    left: i32,
    top: i32,
    line: u32,
}

impl Written {
    fn row_top(&self, row: usize) -> i32 {
        self.top.saturating_add(signed(count(row) * self.line))
    }

    /// The command, what is suggested after it and the cursor.
    fn draw(
        &self,
        model: &Model,
        view: &EditorView,
        command: &wrap::Command,
        measure: &dyn TextMeasure,
        into: &mut DrawList,
    ) {
        let ink = |color| Ink {
            font: UiFont::Mono,
            size: view.letters,
            color,
        };
        for (at, row) in command.rows.iter().enumerate() {
            let place = (self.left, self.row_top(at));
            into.texts
                .push(ink(model.palette.text).run(place, row.clone()));
        }
        let (row, column) = command.cursor;
        let before: String = command
            .rows
            .get(row)
            .map(|text| text.chars().take(column).collect())
            .unwrap_or_default();
        let along = measure.width(&before, UiFont::Mono, view.letters);
        let at = (self.left.saturating_add(signed(along)), self.row_top(row));
        if !command.suggestion.is_empty() {
            let grey = ink(model.palette.suggestion);
            into.texts.push(grey.run(at, command.suggestion.clone()));
        }
        if model.focused {
            // A bar before the next character; it does not blink (T-D6).
            into.rounded.push(Rounded {
                area: Rect {
                    x: at.0,
                    y: at.1,
                    width: metrics::px(metrics::CURSOR_WIDTH, model.scale).max(1),
                    height: self.line,
                },
                radius: 0,
                color: model.palette.accent,
            });
        }
    }
}

#[cfg(test)]
mod tests;
