use std::cell::RefCell;

use super::*;

/// Each letter is a 4×6 square, one pixel right of the pen and five above the
/// baseline, and moves the pen 4.5; a space has no picture.
#[derive(Default)]
struct SquareLetters {
    pictured: RefCell<Vec<u16>>,
}

impl GlyphSource for SquareLetters {
    fn shape(&self, _font: UiFont, character: char, size: u32) -> Option<Shaped> {
        let glyph = u16::try_from(u32::from(character)).ok()?;
        let key = GlyphKey {
            face: 0,
            glyph,
            size,
        };
        (character != '?').then_some(Shaped { key, advance: 4.5 })
    }

    /// The face says the look: 2 plain, 3 bold, 4 italic, 5 both. A mark
    /// above a letter does not move the pen.
    fn shape_grid(&self, character: char, look: GridLook, size: f32) -> Option<Shaped> {
        let glyph = u16::try_from(u32::from(character)).ok()?;
        let key = GlyphKey {
            face: 2 + u32::from(look.bold) + 2 * u32::from(look.italic),
            glyph,
            size: size as u32,
        };
        let mark = ('\u{300}'..='\u{36f}').contains(&character) || character == '\u{20d7}';
        let advance = if mark { 0.0 } else { 4.5 };
        Some(Shaped { key, advance })
    }

    fn picture(&self, key: GlyphKey) -> Result<Picture, String> {
        self.pictured.borrow_mut().push(key.glyph);
        if key.glyph == u16::from(b'!') {
            return Err("it breaks".into());
        }
        let side = |length| if key.glyph == 32 { 0 } else { length };
        // One mark is drawn behind the pen; the rest, ahead as the letters.
        let behind = key.glyph == 0x20D7;
        // The vertical line of the boxes reaches past a cell of 20: 4 above it
        // and 3 below, from a baseline at 15.
        // An integral sign does the same without being of a box.
        let (top, tall) = if [0x2502, 0x222B].contains(&key.glyph) {
            (-19, 27)
        } else {
            (-5, 6)
        };
        Ok(Picture {
            left: if behind { -5 } else { 1 },
            top,
            width: side(4),
            height: side(tall),
            coverage: vec![200; (side(4) * side(tall)) as usize],
        })
    }

    fn ascent(&self, _font: UiFont, size: u32) -> i32 {
        i32::try_from(size).unwrap_or(0)
    }
}

fn run(text: &str) -> TextRun {
    TextRun {
        x: 10,
        y: 20,
        text: text.into(),
        color: Rgb::WHITE,
        font: UiFont::Text,
        size: 8,
    }
}

fn texts(all: &[&str]) -> DrawList {
    DrawList {
        texts: all.iter().map(|text| run(text)).collect(),
        ..DrawList::default()
    }
}

fn filled(list: &DrawList, source: &SquareLetters, atlas: &mut Atlas) -> Batch {
    let mut batch = Batch::default();
    batch.fill(list, None, source, atlas);
    batch
}

#[test]
fn each_rectangle_with_something_to_see_is_one_instance_in_order() {
    let area = |x, width| Rect {
        x,
        y: 2,
        width,
        height: 30,
    };
    let list = DrawList {
        rects: vec![
            (area(1, 20), Rgb(255, 0, 51)),
            (area(5, 0), Rgb::WHITE),
            (area(9, 4), Rgb::BLACK),
        ],
        ..DrawList::default()
    };
    let found = filled(&list, &SquareLetters::default(), &mut Atlas::new(64)).instances;
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].area, [1.0, 2.0, 20.0, 30.0]);
    assert_eq!(found[0].color, [1.0, 0.0, 0.2, 1.0]);
    assert_eq!((found[0].glyph, found[0].atlas), (0.0, [0.0; 4]));
    assert_eq!(found[1].area[0], 9.0);
}

#[test]
fn letters_stand_on_the_baseline_and_start_on_whole_pixels() {
    let list = texts(&["ab c"]);
    let batch = filled(&list, &SquareLetters::default(), &mut Atlas::new(64));
    let areas: Vec<_> = batch.instances.iter().map(|i| i.area).collect();
    // The pen goes 10, 14.5, 19, 23.5; the space is not drawn.
    let at = |x| [x, 23.0, 4.0, 6.0];
    assert_eq!(areas, [at(11.0), at(16.0), at(25.0)]);
    assert!(batch.instances.iter().all(|i| i.glyph == 1.0));
    assert_eq!(batch.instances[1].atlas, [4.0, 0.0, 4.0, 6.0]);
}

#[test]
fn a_glyph_is_pictured_and_copied_to_the_texture_only_once() {
    let source = SquareLetters::default();
    let mut atlas = Atlas::new(64);
    let first = filled(&texts(&["aa", "a "]), &source, &mut atlas);
    assert_eq!(first.uploads.len(), 1);
    assert_eq!(first.uploads[0].coverage.len(), 24);
    assert_eq!(first.instances.len(), 3);
    let second = filled(&texts(&["aa"]), &source, &mut atlas);
    assert!(second.uploads.is_empty());
    assert_eq!(second.instances.len(), 2);
    assert_eq!(*source.pictured.borrow(), [97, 32]);
}

#[test]
fn what_cannot_be_drawn_is_left_out_without_stopping_the_rest() {
    let list = texts(&["?!a"]);
    let batch = filled(&list, &SquareLetters::default(), &mut Atlas::new(64));
    assert_eq!(batch.instances.len(), 1);
    // "?" has no glyph and does not move the pen; "!" has no picture and does.
    assert_eq!(batch.instances[0].area[0], 16.0);
}

#[test]
fn a_full_atlas_is_emptied_and_the_whole_frame_is_made_again() {
    // Room for two letters of 4×6.
    let mut atlas = Atlas::new(8);
    let source = SquareLetters::default();
    filled(&texts(&["ab"]), &source, &mut atlas);
    let batch = filled(&texts(&["cd"]), &source, &mut atlas);
    assert_eq!(batch.instances.len(), 2);
    assert_eq!(batch.uploads.len(), 2);
    let places: Vec<_> = batch.uploads.iter().map(|u| (u.slot.x, u.slot.y)).collect();
    assert_eq!(places, [(0, 0), (4, 0)]);
}

#[test]
fn with_more_glyphs_than_the_atlas_holds_some_are_left_out() {
    let list = texts(&["abc", "a"]);
    let batch = filled(&list, &SquareLetters::default(), &mut Atlas::new(8));
    assert_eq!(batch.instances.len(), 3);
}

mod grid;
