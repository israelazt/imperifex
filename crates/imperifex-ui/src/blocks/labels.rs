//! The labels at the right of a command, from right to left: how it ended, how
//! long it ran and the button that copies.

use imperifex_domain::draw::{DrawList, Icon, Rect, Rounded, UiFont};
use imperifex_domain::i18n::{text, Text};
use imperifex_domain::ports::TextMeasure;
use imperifex_domain::space::Space;
use imperifex_domain::theme::Rgb;
use imperifex_domain::view::{BlockView, CommandRow, Model, RanView};

use super::signed;
use crate::metrics;
use crate::text::Ink;

/// The sizes of the labels in physical pixels.
struct Sizes {
    letters: u32,
    icon: u32,
    gap: u32,
    tight: u32,
    padding: u32,
}

pub struct Labels<'a> {
    row: CommandRow,
    ran: &'a RanView,
    selected: bool,
    /// Its right end.
    end: i32,
    /// All of them, with the room between.
    width: u32,
    button_width: u32,
    /// There is no empty room for them: they go over what is written.
    covers: bool,
    sizes: Sizes,
}

fn sizes(scale: f32) -> Sizes {
    Sizes {
        letters: metrics::px(metrics::SECONDARY_TEXT, scale),
        icon: metrics::px(metrics::RESULT_ICON, scale),
        gap: Space::M.px(scale),
        tight: Space::Xs.px(scale),
        padding: Space::S.px(scale),
    }
}

/// The labels of `block`, if it ran something, its command is in view and they
/// are to be seen: in empty room, or over the text while the mouse is on it.
pub fn place<'a>(
    model: &Model,
    block: &'a BlockView,
    hovered: bool,
    measure: &dyn TextMeasure,
) -> Option<Labels<'a>> {
    let (row, ran) = (block.command?, block.ran.as_ref()?);
    let sizes = sizes(model.scale);
    let words = |said: &str, font| measure.width(said, font, sizes.letters);
    let mut width = sizes.icon;
    if ran.code != 0 {
        width += sizes.tight + words(&ran.code.to_string(), UiFont::Mono);
    }
    if let Some(took) = &ran.took {
        width += sizes.gap + words(took, UiFont::Text);
    }
    let copy = words(text(model.language, Text::CopyOutput), UiFont::Text);
    let button_width = sizes.padding * 2 + sizes.icon + sizes.tight + copy;
    if block.selected {
        width += sizes.gap + button_width;
    }
    // Some room is left between the command and what is said of it.
    let covers = width + sizes.gap > row.free;
    if covers && !hovered {
        return None;
    }
    let side = Space::L.px(model.scale);
    Some(Labels {
        row,
        ran,
        selected: block.selected,
        end: signed(model.window.0.saturating_sub(side)),
        width,
        button_width,
        covers,
        sizes,
    })
}

impl Labels<'_> {
    fn left(&self) -> i32 {
        self.end.saturating_sub(signed(self.width))
    }

    /// Where the button that copies is, on the chosen block.
    pub fn button(&self) -> Option<Rect> {
        let tall = self.row.height + self.sizes.tight;
        self.selected.then(|| Rect {
            x: self.left(),
            y: self.row.y - signed(self.sizes.tight / 2),
            width: self.button_width,
            height: tall,
        })
    }

    pub fn draw(&self, model: &Model, measure: &dyn TextMeasure, into: &mut DrawList) {
        let mut said = DrawList::default();
        self.write(model, measure, &mut said);
        if !self.covers {
            into.rounded.append(&mut said.rounded);
            into.texts.append(&mut said.texts);
            return;
        }
        // Over the text: on a patch of the block's background, above it.
        let behind = if self.selected {
            model.palette.raised
        } else {
            model.palette.surface
        };
        let from = self.left().saturating_sub(signed(self.sizes.gap));
        let patch = Rect {
            x: from,
            y: self.row.y,
            width: u32::try_from(self.end.saturating_sub(from)).unwrap_or(0),
            height: self.row.height,
        };
        into.floating_rects.push((patch, behind));
        let squared = said.rounded.iter().map(|shape| (shape.area, shape.color));
        into.floating_rects.extend(squared);
        into.floating_texts.append(&mut said.texts);
    }

    fn write(&self, model: &Model, measure: &dyn TextMeasure, into: &mut DrawList) {
        let palette = model.palette;
        let (row, sizes) = (&self.row, &self.sizes);
        let strip = |x: i32, width: u32| Rect {
            x,
            y: row.y,
            width,
            height: row.height,
        };
        let ink = |font, size, color: Rgb| Ink { font, size, color };
        let letters = |font, color| ink(font, sizes.letters, color);
        if let Some(button) = self.button() {
            into.rounded.push(Rounded {
                area: button,
                radius: metrics::px(metrics::BLOCK_RADIUS, model.scale),
                color: palette.surface,
            });
            let x = button.x.saturating_add(signed(sizes.padding));
            let glyph = ink(UiFont::Icons, sizes.icon, palette.text);
            glyph.centred(Icon::Copy, strip(x, sizes.icon), measure, into);
            let after = x.saturating_add(signed(sizes.icon + sizes.tight));
            let words = text(model.language, Text::CopyOutput).to_string();
            letters(UiFont::Text, palette.text).beside(
                after,
                strip(after, 0),
                words,
                measure,
                into,
            );
        }
        let failed = self.ran.code != 0;
        let mut x = self.end;
        let mut color = palette.ok;
        if failed {
            color = palette.error;
            let code = self.ran.code.to_string();
            let number = letters(UiFont::Mono, color);
            x = x.saturating_sub(signed(measure.width(&code, number.font, number.size)));
            number.beside(x, strip(x, 0), code, measure, into);
            x = x.saturating_sub(signed(sizes.tight));
        }
        x = x.saturating_sub(signed(sizes.icon));
        let result = if failed { Icon::Cross } else { Icon::Check };
        let glyph = ink(UiFont::Icons, sizes.icon, color);
        glyph.centred(result, strip(x, sizes.icon), measure, into);
        if let Some(took) = &self.ran.took {
            let grey = letters(UiFont::Text, palette.muted);
            let width = measure.width(took, grey.font, grey.size);
            x = x.saturating_sub(signed(width + sizes.gap));
            grey.beside(x, strip(x, width), took.clone(), measure, into);
        }
    }
}
