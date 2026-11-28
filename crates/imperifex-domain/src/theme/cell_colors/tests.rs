use super::*;
use crate::theme::{palette, Theme};

fn dark() -> &'static Palette {
    palette(Theme::Dark)
}

fn style(foreground: Color, background: Color, attributes: Attributes) -> Style {
    Style {
        foreground,
        background,
        attributes,
    }
}

/// What a text colour becomes on the default background, contrast untouched.
fn text_of(colour: Color) -> Rgb {
    let plain = style(colour, Color::Default, Attributes::default());
    cell_colors(plain, dark(), false).0
}

#[test]
fn the_default_colours_are_the_themes_text_on_its_surface() {
    let colours = cell_colors(Style::default(), dark(), true);
    assert_eq!(colours, (dark().text, dark().surface));
}

#[test]
fn the_first_16_numbers_come_from_the_theme() {
    assert_eq!(text_of(Color::Indexed(1)), dark().ansi[1]);
    assert_eq!(text_of(Color::Indexed(15)), dark().ansi[15]);
}

#[test]
fn the_other_numbers_are_xterms_cube_and_greys_and_rgb_is_kept() {
    assert_eq!(text_of(Color::Indexed(16)), Rgb(0, 0, 0));
    assert_eq!(text_of(Color::Indexed(21)), Rgb(0, 0, 255));
    assert_eq!(text_of(Color::Indexed(110)), Rgb(135, 175, 215));
    assert_eq!(text_of(Color::Indexed(231)), Rgb(255, 255, 255));
    assert_eq!(text_of(Color::Indexed(232)), Rgb(8, 8, 8));
    assert_eq!(text_of(Color::Indexed(255)), Rgb(238, 238, 238));
    assert_eq!(text_of(Color::Rgb(1, 2, 3)), Rgb(1, 2, 3));
}

#[test]
fn inverse_swaps_text_and_background() {
    let inverse = style(Color::Default, Color::Default, Attributes::INVERSE);
    let colours = cell_colors(inverse, dark(), true);
    assert_eq!(colours, (dark().surface, dark().text));
}

#[test]
fn dim_text_is_halfway_to_the_background() {
    let dim = style(
        Color::Rgb(200, 100, 0),
        Color::Rgb(0, 0, 100),
        Attributes::DIM,
    );
    let (text, _) = cell_colors(dim, dark(), false);
    assert_eq!(text, Rgb(100, 50, 50));
}

#[test]
fn hidden_text_stays_hidden_even_with_the_minimum_contrast() {
    let attributes = Attributes::HIDDEN.with(Attributes::DIM);
    let hidden = style(Color::Indexed(1), Color::Indexed(4), attributes);
    let (text, background) = cell_colors(hidden, dark(), true);
    assert_eq!(text, background);
    assert_eq!(background, dark().ansi[4]);
}

#[test]
fn the_suggestion_of_psreadline_is_seen_on_the_light_theme() {
    // `ESC[97;2;3m`: bright white, dim and italic.
    let suggestion = style(Color::Indexed(15), Color::Default, Attributes::DIM);
    let light = palette(Theme::Light);
    let (unadjusted, _) = cell_colors(suggestion, light, false);
    assert_eq!(unadjusted, Rgb::WHITE);
    let (text, background) = cell_colors(suggestion, light, true);
    assert!(contrast(text, background) >= ENOUGH);
    // Only as far as needed: it is still a faint grey, not black.
    assert!(contrast(text, background) < ENOUGH + 0.1);
}

#[test]
fn a_pair_that_can_already_be_told_apart_is_not_changed() {
    let (text, background) = (Rgb(119, 119, 119), Rgb(60, 60, 60));
    let found = contrast(text, background);
    assert!((TOO_LOW..ENOUGH).contains(&found), "{found}");
    let pair = style(
        Color::Rgb(119, 119, 119),
        Color::Rgb(60, 60, 60),
        Attributes::default(),
    );
    assert_eq!(cell_colors(pair, dark(), true), (text, background));
}

/// A grid of colours all over the RGB cube.
fn samples() -> Vec<Color> {
    let steps = [0u8, 51, 102, 153, 204, 255];
    let mut colours = Vec::new();
    for r in steps {
        for g in steps {
            colours.extend(steps.map(|b| Color::Rgb(r, g, b)));
        }
    }
    colours
}

#[test]
fn any_text_on_any_background_ends_up_visible() {
    for background in samples() {
        for foreground in samples() {
            let pair = style(foreground, background, Attributes::default());
            let (text, behind) = cell_colors(pair, dark(), true);
            assert!(contrast(text, behind) >= TOO_LOW, "{pair:?}");
        }
    }
}
