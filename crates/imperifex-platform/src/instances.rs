//! What the graphics card draws: one instance per rectangle or glyph, which the
//! vertex shader opens into its four corners. Portable, so that it is tested
//! everywhere: the letters come from a [`GlyphSource`].

mod grid;

use imperifex_domain::atlas::{Atlas, GlyphKey, Slot};
use imperifex_domain::draw::{DrawList, Rect, Rounded, TextRun, UiFont};
use imperifex_domain::log_info;
use imperifex_domain::picture::TerminalPicture;
use imperifex_domain::theme::Rgb;

/// Laid out as `shaders.hlsl` reads it.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Instance {
    /// Left, top, width and height in physical pixels.
    pub area: [f32; 4],
    /// Red, green, blue and alpha, from 0 to 1.
    pub color: [f32; 4],
    /// For a glyph, where it is in the atlas: left, top, width and height.
    pub atlas: [f32; 4],
    /// 1 for a glyph, whose shape comes from the atlas; 0 for a rectangle.
    pub glyph: f32,
    /// How round the corners of a rectangle are, in pixels; 0 for square ones.
    pub radius: f32,
}

/// The glyph a character is drawn with, and how far it moves the pen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shaped {
    pub key: GlyphKey,
    pub advance: f32,
}

/// The picture of a glyph: how much of each pixel it covers, row by row.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Picture {
    /// From the pen to its left side, and from the baseline to its top.
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    pub coverage: Vec<u8>,
}

/// The letters of the interface and of the grid, as the system gives them.
pub trait GlyphSource {
    /// `size` is that of the letters, in physical pixels. `None` when nothing
    /// can draw the character.
    fn shape(&self, font: UiFont, character: char, size: u32) -> Option<Shaped>;
    /// The same in the font of the grid, with letters of `size` pixels.
    fn shape_grid(&self, character: char, look: GridLook, size: f32) -> Option<Shaped>;
    /// The picture of a glyph that [`GlyphSource::shape`] gave.
    fn picture(&self, key: GlyphKey) -> Result<Picture, String>;
    /// From the top of a line down to its baseline.
    fn ascent(&self, font: UiFont, size: u32) -> i32;
}

/// Which of the grid's faces a cell is written with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GridLook {
    pub bold: bool,
    pub italic: bool,
}

/// A picture to copy to the texture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upload {
    pub slot: Slot,
    pub coverage: Vec<u8>,
}

/// One frame ready for the card. It is kept to use its memory again.
#[derive(Debug, Default)]
pub struct Batch {
    pub instances: Vec<Instance>,
    /// What the texture does not have yet.
    pub uploads: Vec<Upload>,
    /// The rows of the terminal as they were last made.
    grid: grid::Rows,
}

/// The atlas had no room for a glyph.
struct Full;

impl Batch {
    /// The rectangles of `list` in order, the grid of `terminal`, the texts of
    /// the list and what floats, each over what came before. If the atlas
    /// fills up it is emptied and the frame is made again, so that every glyph
    /// of a frame is in it; with more of them than it holds, some are left out.
    pub fn fill(
        &mut self,
        list: &DrawList,
        terminal: Option<&TerminalPicture<'_>>,
        source: &dyn GlyphSource,
        atlas: &mut Atlas,
    ) {
        if self.try_fill(list, terminal, source, atlas).is_err() {
            log_info!("the glyph atlas is full: it is emptied");
            atlas.clear();
            if self.try_fill(list, terminal, source, atlas).is_err() {
                log_info!("a frame has more glyphs than the atlas holds");
                // The rows that lack glyphs are not to be kept.
                self.grid.forget();
            }
        }
    }

    /// Goes on past a glyph that does not fit, and says so at the end.
    fn try_fill(
        &mut self,
        list: &DrawList,
        terminal: Option<&TerminalPicture<'_>>,
        source: &dyn GlyphSource,
        atlas: &mut Atlas,
    ) -> Result<(), Full> {
        self.instances.clear();
        self.uploads.clear();
        self.solids(&list.rects);
        self.instances.extend(list.rounded.iter().map(round));
        let mut letters = Letters {
            source,
            atlas,
            uploads: &mut self.uploads,
        };
        let mut result = match terminal {
            Some(picture) => self.grid.draw(picture, &mut letters, &mut self.instances),
            None => Ok(()),
        };
        for run in &list.texts {
            result = letters.text(run, &mut self.instances).and(result);
        }
        self.solids(&list.floating_rects);
        let mut letters = Letters {
            source,
            atlas,
            uploads: &mut self.uploads,
        };
        for run in &list.floating_texts {
            result = letters.text(run, &mut self.instances).and(result);
        }
        result
    }

    fn solids(&mut self, rects: &[(Rect, Rgb)]) {
        let visible = rects
            .iter()
            .filter(|(area, _)| area.width > 0 && area.height > 0);
        self.instances
            .extend(visible.map(|&(area, color)| solid(area, color)));
    }
}

/// Where glyphs come from and where their pictures are kept.
struct Letters<'a> {
    source: &'a dyn GlyphSource,
    atlas: &'a mut Atlas,
    uploads: &'a mut Vec<Upload>,
}

impl Letters<'_> {
    fn text(&mut self, run: &TextRun, into: &mut Vec<Instance>) -> Result<(), Full> {
        let ascent = self.source.ascent(run.font, run.size);
        let baseline = run.y.saturating_add(ascent);
        // A place on a screen is far inside what an `f32` holds exactly.
        let mut pen = run.x as f32;
        let mut result = Ok(());
        for character in run.text.chars() {
            let Some(shaped) = self.source.shape(run.font, character, run.size) else {
                continue;
            };
            // Each glyph starts on a whole pixel, or it would be blurred.
            let at = (pen.round() as i32, baseline);
            result = self.glyph(shaped.key, at, run.color, into).and(result);
            pen += shaped.advance;
        }
        result
    }

    /// Adds the glyph with the pen at `pen`, on the baseline given.
    fn glyph(
        &mut self,
        key: GlyphKey,
        (pen, baseline): (i32, i32),
        color: Rgb,
        into: &mut Vec<Instance>,
    ) -> Result<(), Full> {
        let slot = self.slot(key)?;
        if slot.width == 0 || slot.height == 0 {
            return Ok(());
        }
        let area = Rect {
            x: pen.saturating_add(slot.left),
            y: baseline.saturating_add(slot.top),
            width: slot.width,
            height: slot.height,
        };
        into.push(Instance {
            atlas: [
                slot.x as f32,
                slot.y as f32,
                slot.width as f32,
                slot.height as f32,
            ],
            glyph: 1.0,
            ..solid(area, color)
        });
        Ok(())
    }

    /// Where the glyph is in the atlas, putting it there if it is not. One that
    /// cannot be pictured takes an empty place, so as not to ask again.
    fn slot(&mut self, key: GlyphKey) -> Result<Slot, Full> {
        if let Some(slot) = self.atlas.get(key) {
            return Ok(slot);
        }
        let picture = self.source.picture(key).unwrap_or_else(|error| {
            log_info!("a glyph could not be drawn: {error}");
            Picture::default()
        });
        let wanted = Slot {
            width: picture.width,
            height: picture.height,
            left: picture.left,
            top: picture.top,
            ..Slot::default()
        };
        let slot = self.atlas.insert(key, wanted).ok_or(Full)?;
        if slot.width > 0 && slot.height > 0 {
            self.uploads.push(Upload {
                slot,
                coverage: picture.coverage,
            });
        }
        Ok(slot)
    }
}

fn round(shape: &Rounded) -> Instance {
    // No rounder than half its shorter side.
    let most = shape.area.width.min(shape.area.height) / 2;
    Instance {
        radius: shape.radius.min(most) as f32,
        ..solid(shape.area, shape.color)
    }
}

fn solid(area: Rect, color: Rgb) -> Instance {
    let channel = |value: u8| f32::from(value) / 255.0;
    // A place on a screen is far inside what an `f32` holds exactly.
    Instance {
        area: [
            area.x as f32,
            area.y as f32,
            area.width as f32,
            area.height as f32,
        ],
        color: [channel(color.0), channel(color.1), channel(color.2), 1.0],
        ..Instance::default()
    }
}

#[cfg(test)]
mod tests;
