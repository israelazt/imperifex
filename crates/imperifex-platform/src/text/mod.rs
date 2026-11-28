//! The letters of Windows, with DirectWrite (T-D14): it measures them for the
//! interface and pictures them for the renderer.

mod face;
mod fallback;
mod raster;

use std::cell::RefCell;
use std::collections::HashMap;

use imperifex_domain::atlas::GlyphKey;
use imperifex_domain::draw::{Icon, UiFont};
use imperifex_domain::font::{cell_size, CellSize};
use imperifex_domain::log_warn;
use imperifex_domain::ports::TextMeasure;
use imperifex_domain::prompt_icon;
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory2, IDWriteFontCollection, DWRITE_FACTORY_TYPE_SHARED,
};

use crate::instances::{GlyphSource, GridLook, Picture, Shaped};
use face::{Face, Look};

/// The font of the interface's words: the one of Windows 11, or else the one of
/// Windows 10.
const WORDS: [&str; 2] = ["Segoe UI Variable Text", "Segoe UI"];
/// That of its icons: the one of Windows 11, or else the one of Windows 10.
const ICONS: [&str; 2] = ["Segoe Fluent Icons", "Segoe MDL2 Assets"];
/// What the grid uses when the chosen font is not installed.
const SPARE: &str = "Consolas";

/// Where each face is in the list: its number in a [`GlyphKey`].
const WORDS_FACE: u32 = 0;
const ICONS_FACE: u32 = 1;
/// Not a face of Windows: what is drawn by the program itself.
const DRAWN_FACE: u32 = u32::MAX;
/// The first of the grid's four, in the order of [`Look::ALL`].
const GRID_FACES: u32 = 2;

/// For a character and the face that does not have it: the face that stands
/// in, and by how much to scale its letters; `None` when no font has it.
type StandIns = HashMap<(char, u32), Option<(u32, f32)>>;

pub struct DirectWriteText {
    factory: IDWriteFactory2,
    collection: IDWriteFontCollection,
    /// The first ones are fixed; those of fallback are added as needed.
    faces: RefCell<Vec<Face>>,
    stand_ins: RefCell<StandIns>,
    found: bool,
}

impl DirectWriteText {
    /// With `grid_family` for the grid, or Consolas if it is not installed:
    /// [`DirectWriteText::found_grid_font`] says which.
    pub fn new(grid_family: &str) -> Result<Self, String> {
        // SAFETY: it only takes the kind of factory.
        let factory: IDWriteFactory2 = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED) }
            .map_err(failed("DWriteCreateFactory"))?;
        let collection = face::system_fonts(&factory)?;
        let wanted = face::family(&collection, grid_family)?;
        let found = wanted.is_some();
        let grid = match wanted {
            Some(family) => family,
            None => face::installed(&collection, &[SPARE])?,
        };
        let words = face::installed(&collection, &WORDS)?;
        let icons = face::installed(&collection, &ICONS)?;
        let mut faces = vec![
            Face::of(&words, Look::REGULAR)?,
            Face::of(&icons, Look::REGULAR)?,
        ];
        for look in Look::ALL {
            faces.push(Face::of(&grid, look)?);
        }
        Ok(Self {
            factory,
            collection,
            faces: RefCell::new(faces),
            stand_ins: RefCell::default(),
            found,
        })
    }

    pub fn found_grid_font(&self) -> bool {
        self.found
    }

    /// The cell of the grid for letters of `points` at the window's `scale`.
    pub fn cell(&self, points: f32, scale: f32) -> Result<CellSize, String> {
        let faces = self.faces.borrow();
        let metrics = face_at(&faces, GRID_FACES)?.metrics;
        Ok(cell_size(&metrics, points, scale))
    }

    /// The glyph of `character` in the face `wanted`, or in one that stands in
    /// for it when that one does not have it.
    fn shape_in(&self, wanted: u32, character: char, size: f32) -> Result<Shaped, String> {
        let has = face_at(&self.faces.borrow(), wanted)?.glyph(character)?;
        let (number, size) = match has {
            0 if wanted != ICONS_FACE => match self.stand_in(wanted, character) {
                Some((other, scale)) => (other, size * scale),
                None => (wanted, size),
            },
            _ => (wanted, size),
        };
        let faces = self.faces.borrow();
        let face = face_at(&faces, number)?;
        let glyph = face.glyph(character)?;
        Ok(Shaped {
            key: GlyphKey {
                face: number,
                glyph,
                // In 64ths of a pixel; never negative, so the cast cannot wrap.
                size: (size * 64.0).round().max(0.0) as u32,
            },
            advance: face.advance(glyph, size)?,
        })
    }

    /// Asked of Windows once for each character and face.
    fn stand_in(&self, wanted: u32, character: char) -> Option<(u32, f32)> {
        if let Some(known) = self.stand_ins.borrow().get(&(character, wanted)) {
            return *known;
        }
        let found = self
            .ask_for_stand_in(wanted, character)
            .map_err(|error| log_warn!("no other font for {character:?}: {error}"))
            .ok()
            .flatten();
        self.stand_ins
            .borrow_mut()
            .insert((character, wanted), found);
        found
    }

    fn ask_for_stand_in(&self, wanted: u32, character: char) -> Result<Option<(u32, f32)>, String> {
        let look = Look::of_number(wanted);
        let Some((face, scale)) =
            fallback::face_for(&self.factory, &self.collection, character, look)?
        else {
            return Ok(None);
        };
        let mut faces = self.faces.borrow_mut();
        // The same font standing in for another character is the same face.
        let number = match faces.iter().position(|other| other.is_same(&face)) {
            Some(number) => number,
            None => {
                faces.push(face);
                faces.len() - 1
            }
        };
        let number = u32::try_from(number).map_err(|_| "too many fonts".to_string())?;
        Ok(Some((number, scale)))
    }

    /// What there is to know of the face of one of the interface's fonts.
    fn of_ui(&self, font: UiFont) -> Option<imperifex_domain::font::FontMetrics> {
        let faces = self.faces.borrow();
        face_at(&faces, number_of(font)).ok().map(|f| f.metrics)
    }
}

fn number_of(font: UiFont) -> u32 {
    match font {
        UiFont::Text => WORDS_FACE,
        UiFont::Icons => ICONS_FACE,
    }
}

fn face_at(faces: &[Face], number: u32) -> Result<&Face, String> {
    usize::try_from(number)
        .ok()
        .and_then(|at| faces.get(at))
        .ok_or_else(|| format!("there is no face {number}"))
}

/// The icon of a shell: as wide as its size, like those of the icon font.
fn drawn_prompt(size: u32) -> Shaped {
    Shaped {
        key: GlyphKey {
            face: DRAWN_FACE,
            glyph: 0,
            size: size.saturating_mul(64),
        },
        advance: size as f32,
    }
}

impl GlyphSource for DirectWriteText {
    fn shape(&self, font: UiFont, character: char, size: u32) -> Option<Shaped> {
        if font == UiFont::Icons && character == Icon::Shell.character() {
            return Some(drawn_prompt(size));
        }
        self.shape_in(number_of(font), character, size as f32)
            .map_err(|error| log_warn!("{character:?} cannot be drawn: {error}"))
            .ok()
    }

    fn shape_grid(&self, character: char, look: GridLook, size: f32) -> Option<Shaped> {
        // In the order of [`Look::ALL`].
        let face = GRID_FACES + u32::from(look.bold) + 2 * u32::from(look.italic);
        self.shape_in(face, character, size)
            .map_err(|error| log_warn!("{character:?} cannot be drawn: {error}"))
            .ok()
    }

    fn picture(&self, key: GlyphKey) -> Result<Picture, String> {
        if key.face == DRAWN_FACE {
            let side = key.size / 64;
            let coverage = prompt_icon::coverage(side);
            return Ok(Picture {
                left: 0,
                // Its top where that of the icon font's line is.
                top: -self.ascent(UiFont::Icons, side),
                width: side,
                height: side,
                coverage,
            });
        }
        let faces = self.faces.borrow();
        let face = face_at(&faces, key.face)?;
        raster::picture(&self.factory, face, key.glyph, key.size as f32 / 64.0)
    }

    fn ascent(&self, font: UiFont, size: u32) -> i32 {
        self.of_ui(font).map_or(0, |metrics| {
            let ascent = metrics.pixels(f32::from(metrics.ascent), size as f32);
            ascent.round() as i32
        })
    }
}

impl TextMeasure for DirectWriteText {
    fn width(&self, text: &str, font: UiFont, size: u32) -> u32 {
        let advances: f32 = text
            .chars()
            .filter_map(|character| self.shape(font, character, size))
            .map(|shaped| shaped.advance)
            .sum();
        // A sum of advances: never negative.
        advances.ceil() as u32
    }

    fn line_height(&self, font: UiFont, size: u32) -> u32 {
        self.of_ui(font).map_or(0, |metrics| {
            let line = metrics.pixels(metrics.line_units(), size as f32);
            line.ceil().max(0.0) as u32
        })
    }
}

/// Says which call an error came from.
fn failed(call: &'static str) -> impl Fn(windows::core::Error) -> String {
    move |error| format!("{call}: {error}")
}

/// What a call that went well was to give.
fn created<T>(object: Option<T>) -> Result<T, String> {
    object.ok_or_else(|| "DirectWrite gave no object".into())
}

#[cfg(test)]
mod tests;
