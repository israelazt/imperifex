//! How well two colours are told apart, as WCAG 2 measures it.

use super::Rgb;

/// The relative luminance, from 0 (black) to 1 (white).
pub fn luminance(colour: Rgb) -> f32 {
    let linear = |channel: u8| {
        let value = f32::from(channel) / 255.0;
        if value <= 0.040_45 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(colour.0) + 0.7152 * linear(colour.1) + 0.0722 * linear(colour.2)
}

/// From 1 (the same) to 21 (black on white). The order does not matter.
pub fn contrast(a: Rgb, b: Rgb) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{palette, Palette, Theme};

    /// The lowest contrast of `colour` over the three backgrounds.
    fn worst(palette: &Palette, colour: Rgb) -> f32 {
        [palette.background, palette.surface, palette.raised]
            .map(|background| contrast(colour, background))
            .into_iter()
            .fold(f32::MAX, f32::min)
    }

    #[test]
    fn black_on_white_is_21_and_a_colour_on_itself_is_1() {
        assert!((contrast(Rgb::BLACK, Rgb::WHITE) - 21.0).abs() < 0.001);
        assert!((contrast(Rgb::WHITE, Rgb::BLACK) - 21.0).abs() < 0.001);
        assert!((contrast(Rgb(90, 20, 200), Rgb(90, 20, 200)) - 1.0).abs() < 0.001);
    }

    #[test]
    fn every_text_colour_reads_on_the_three_backgrounds_of_its_theme() {
        for theme in Theme::ALL {
            let p = palette(theme);
            for colour in [
                p.text, p.muted, p.accent, p.ok, p.warning, p.error, p.waiting,
            ] {
                assert!(
                    worst(p, colour) >= 4.5,
                    "{theme:?} {colour:?} {}",
                    worst(p, colour)
                );
            }
        }
    }

    #[test]
    fn the_ansi_ones_read_and_the_grey_one_can_be_made_out() {
        for theme in Theme::ALL {
            let p = palette(theme);
            // 0 is "black", used above all as a background.
            for index in (1..=7).chain(9..=15) {
                let found = worst(p, p.ansi[index]);
                assert!(found >= 4.5, "{theme:?} ansi {index}: {found}");
            }
            let grey = worst(p, p.ansi[8]);
            assert!(grey >= 4.0, "{theme:?} ansi 8: {grey}");
        }
    }

    #[test]
    fn what_the_editor_suggests_reads_in_its_box() {
        for theme in Theme::ALL {
            let p = palette(theme);
            let found = contrast(p.suggestion, p.raised);
            assert!(found >= 4.5, "{theme:?}: {found}");
        }
    }

    #[test]
    fn the_icon_that_closes_reads_on_its_red() {
        for theme in Theme::ALL {
            let p = palette(theme);
            let found = contrast(p.on_close_hover, p.close_hover);
            assert!(found >= 4.5, "{theme:?}: {found}");
        }
    }
}
