//! Against the fonts installed in Windows, only with those that every Windows
//! has. No window and no graphics card.

use imperifex_domain::draw::Icon;

use super::*;

fn text_with(family: &str) -> DirectWriteText {
    DirectWriteText::new(family).expect("DirectWrite and the fonts of Windows")
}

fn letter(text: &DirectWriteText, character: char) -> Picture {
    let shaped = text.shape(UiFont::Text, character, 32).expect("a glyph");
    text.picture(shaped.key).expect("its picture")
}

#[test]
fn a_font_that_is_not_installed_gives_the_spare_one() {
    let consolas = text_with("Consolas");
    assert!(consolas.found_grid_font());
    let spare = text_with("No Such Font 8d2f");
    assert!(!spare.found_grid_font());
    assert_eq!(spare.cell(12.0, 1.0), consolas.cell(12.0, 1.0));
}

#[test]
fn the_cell_of_consolas_grows_with_the_scale() {
    let text = text_with("Consolas");
    let at = |scale| text.cell(12.0, scale).expect("a cell");
    let (one, half_more) = (at(1.0), at(1.5));
    assert!(one.width > 5 && one.width < one.height, "{one:?}");
    assert!(one.baseline > 0 && one.baseline < one.height, "{one:?}");
    assert!(half_more.width > one.width && half_more.height > one.height);
}

#[test]
fn a_letter_has_a_picture_above_the_baseline_and_a_space_has_none() {
    let text = text_with("Consolas");
    let a = letter(&text, 'A');
    assert!(a.width > 5 && a.height > 10, "{} {}", a.width, a.height);
    assert_eq!(a.coverage.len(), (a.width * a.height) as usize);
    assert!(a.top < 0 && a.top + a.height as i32 <= 1, "{}", a.top);
    // Solid inside and smoothed at its edges.
    assert!(a.coverage.iter().any(|&c| c > 250));
    assert!(a.coverage.iter().any(|&c| c > 0 && c < 250));
    assert_eq!(letter(&text, ' '), Picture::default());
}

#[test]
fn words_measure_the_sum_of_their_letters_and_grow_with_the_scale() {
    let text = text_with("Consolas");
    let width = |words, scale| text.width(words, UiFont::Text, scale);
    assert_eq!(width("", 1.0), 0);
    assert!(width("W", 1.0) > width("i", 1.0));
    assert!(width("Minimizar", 1.0) > width("Cerrar", 1.0));
    assert!(width("Cerrar", 2.0) > width("Cerrar", 1.0));
    let line = text.line_height(UiFont::Text, 1.0);
    assert!((12..=24).contains(&line), "{line}");
    let ascent = text.ascent(UiFont::Text, 12);
    assert!(ascent > 0 && ascent < line as i32, "{ascent}");
}

#[test]
fn every_icon_is_in_the_icon_font() {
    let text = text_with("Consolas");
    for icon in Icon::ALL {
        let shaped = text
            .shape(UiFont::Icons, icon.character(), 10)
            .expect("a glyph");
        assert_eq!(shaped.key.face, ICONS_FACE);
        assert_ne!(shaped.key.glyph, 0, "{icon:?}");
        let picture = text.picture(shaped.key).expect("its picture");
        assert!(picture.width > 0, "{icon:?}");
    }
}

#[test]
fn a_character_the_font_does_not_have_comes_from_another_one() {
    let text = text_with("Consolas");
    let shaped = text.shape(UiFont::Text, '漢', 32).expect("a glyph");
    assert!(shaped.key.face >= GRID_FACES + 4, "{shaped:?}");
    assert_ne!(shaped.key.glyph, 0);
    assert!(text.picture(shaped.key).expect("its picture").width > 10);
    // Another character of the same font is the same face, not one more.
    let again = text.shape(UiFont::Text, '字', 32).expect("a glyph");
    assert_eq!(again.key.face, shaped.key.face);
}

#[test]
fn bold_and_italic_are_faces_that_draw_differently() {
    let text = text_with("Consolas");
    let pictures: Vec<_> = (0..4)
        .map(|look| {
            let shaped = text
                .shape_in(GRID_FACES + look, 'a', 32.0)
                .expect("a glyph");
            text.picture(shaped.key).expect("its picture")
        })
        .collect();
    for (at, picture) in pictures.iter().enumerate() {
        assert!(!pictures[at + 1..].contains(picture), "{at}");
    }
}
