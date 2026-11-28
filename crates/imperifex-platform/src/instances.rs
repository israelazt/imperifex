//! What the graphics card draws: one instance per rectangle or glyph, which the
//! vertex shader opens into its four corners. Portable, so that it is tested
//! everywhere: the letters come from a [`GlyphSource`].

use imperifex_domain::atlas::{Atlas, GlyphKey, Slot};
use imperifex_domain::draw::{DrawList, Rect, TextRun, UiFont};
use imperifex_domain::log_info;
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

/// The letters of the interface, as the system gives them.
pub trait GlyphSource {
    /// `size` is that of the letters, in physical pixels. `None` when nothing
    /// can draw the character.
    fn shape(&self, font: UiFont, character: char, size: u32) -> Option<Shaped>;
    /// The picture of a glyph that [`GlyphSource::shape`] gave.
    fn picture(&self, key: GlyphKey) -> Result<Picture, String>;
    /// From the top of a line down to its baseline.
    fn ascent(&self, font: UiFont, size: u32) -> i32;
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
}

/// The atlas had no room for a glyph.
struct Full;

impl Batch {
    /// The rectangles of `list` in order, and its texts over them. If the atlas
    /// fills up it is emptied and the frame is made again, so that every glyph
    /// of a frame is in it; with more of them than it holds, some are left out.
    pub fn fill(&mut self, list: &DrawList, source: &dyn GlyphSource, atlas: &mut Atlas) {
        if self.try_fill(list, source, atlas).is_err() {
            log_info!("the glyph atlas is full: it is emptied");
            atlas.clear();
            if self.try_fill(list, source, atlas).is_err() {
                log_info!("a frame has more glyphs than the atlas holds");
            }
        }
    }

    /// Goes on past a glyph that does not fit, and says so at the end.
    fn try_fill(
        &mut self,
        list: &DrawList,
        source: &dyn GlyphSource,
        atlas: &mut Atlas,
    ) -> Result<(), Full> {
        self.instances.clear();
        self.uploads.clear();
        let visible = list
            .rects
            .iter()
            .filter(|(area, _)| area.width > 0 && area.height > 0);
        self.instances
            .extend(visible.map(|&(area, color)| solid(area, color)));
        let mut result = Ok(());
        for run in &list.texts {
            result = self.text(run, source, atlas).and(result);
        }
        result
    }

    fn text(
        &mut self,
        run: &TextRun,
        source: &dyn GlyphSource,
        atlas: &mut Atlas,
    ) -> Result<(), Full> {
        let baseline = run.y.saturating_add(source.ascent(run.font, run.size));
        // A place on a screen is far inside what an `f32` holds exactly.
        let mut pen = run.x as f32;
        let mut result = Ok(());
        for character in run.text.chars() {
            let Some(shaped) = source.shape(run.font, character, run.size) else {
                continue;
            };
            match self.slot(shaped.key, source, atlas) {
                // Each glyph starts on a whole pixel, or it would be blurred.
                Ok(slot) => self.glyph(slot, (pen.round() as i32, baseline), run.color),
                Err(full) => result = Err(full),
            }
            pen += shaped.advance;
        }
        result
    }

    /// Where the glyph is in the atlas, putting it there if it is not. One that
    /// cannot be pictured takes an empty place, so as not to ask again.
    fn slot(
        &mut self,
        key: GlyphKey,
        source: &dyn GlyphSource,
        atlas: &mut Atlas,
    ) -> Result<Slot, Full> {
        if let Some(slot) = atlas.get(key) {
            return Ok(slot);
        }
        let picture = source.picture(key).unwrap_or_else(|error| {
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
        let slot = atlas.insert(key, wanted).ok_or(Full)?;
        if slot.width > 0 && slot.height > 0 {
            self.uploads.push(Upload {
                slot,
                coverage: picture.coverage,
            });
        }
        Ok(slot)
    }

    fn glyph(&mut self, slot: Slot, (pen, baseline): (i32, i32), color: Rgb) {
        if slot.width == 0 || slot.height == 0 {
            return;
        }
        let area = Rect {
            x: pen.saturating_add(slot.left),
            y: baseline.saturating_add(slot.top),
            width: slot.width,
            height: slot.height,
        };
        self.instances.push(Instance {
            atlas: [
                slot.x as f32,
                slot.y as f32,
                slot.width as f32,
                slot.height as f32,
            ],
            glyph: 1.0,
            ..solid(area, color)
        });
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
