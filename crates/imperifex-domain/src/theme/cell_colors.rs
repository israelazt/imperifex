//! The colours a cell is drawn with: what the program asked for, put through the
//! theme, the attributes that change colour and the minimum contrast (T-D17).

use super::contrast::contrast;
use super::{Palette, Rgb};
use crate::grid::{Attributes, Color, Style};

/// Below this contrast the text of a cell is moved away from its background...
const TOO_LOW: f32 = 2.0;
/// ...until it reaches this one.
const ENOUGH: f32 = 3.0;

/// The colour of the text and of the background of a cell with `style`.
/// With `adjust_contrast`, text that would not be seen is made visible; hidden
/// text stays hidden.
pub fn cell_colors(style: Style, palette: &Palette, adjust_contrast: bool) -> (Rgb, Rgb) {
    let has = |attribute| style.attributes.has(attribute);
    let mut text = resolve(style.foreground, palette.text, palette);
    let mut background = resolve(style.background, palette.surface, palette);
    if has(Attributes::INVERSE) {
        std::mem::swap(&mut text, &mut background);
    }
    if has(Attributes::HIDDEN) {
        return (background, background);
    }
    if has(Attributes::DIM) {
        text = mix(text, background, 128);
    }
    if adjust_contrast && contrast(text, background) < TOO_LOW {
        text = made_visible(text, background);
    }
    (text, background)
}

fn resolve(colour: Color, default: Rgb, palette: &Palette) -> Rgb {
    match colour {
        Color::Default => default,
        Color::Indexed(index) => indexed(index, palette),
        Color::Rgb(r, g, b) => Rgb(r, g, b),
    }
}

/// The 256 numbered colours: the theme's 16, then xterm's cube and its greys.
fn indexed(index: u8, palette: &Palette) -> Rgb {
    match index {
        0..=15 => palette.ansi[usize::from(index)],
        16..=231 => {
            let level = |step: u8| if step == 0 { 0 } else { 55 + 40 * step };
            let n = index - 16;
            Rgb(level(n / 36), level(n / 6 % 6), level(n % 6))
        }
        232..=255 => {
            let grey = 8 + 10 * (index - 232);
            Rgb(grey, grey, grey)
        }
    }
}

/// `from` moved toward `to`: not at all with 0, all the way with 255.
fn mix(from: Rgb, to: Rgb, amount: u8) -> Rgb {
    let channel = |a: u8, b: u8| {
        let (a, b, t) = (u32::from(a), u32::from(b), u32::from(amount));
        let mixed = (a * (255 - t) + b * t + 127) / 255;
        u8::try_from(mixed).unwrap_or(u8::MAX)
    };
    Rgb(
        channel(from.0, to.0),
        channel(from.1, to.1),
        channel(from.2, to.2),
    )
}

/// `text` moved toward white or black, whichever stands out more on
/// `background`, only as far as needed. Both give at least 4.5 on any colour,
/// so the goal is always reached.
fn made_visible(text: Rgb, background: Rgb) -> Rgb {
    let target = if contrast(Rgb::WHITE, background) >= contrast(Rgb::BLACK, background) {
        Rgb::WHITE
    } else {
        Rgb::BLACK
    };
    // The text starts below `TOO_LOW`: on the way to the target the contrast may
    // first fall, but once it is enough it stays enough. So it can be halved.
    let (mut low, mut high) = (0u8, u8::MAX);
    while low < high {
        let middle = low + (high - low) / 2;
        if contrast(mix(text, target, middle), background) >= ENOUGH {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    mix(text, target, high)
}

#[cfg(test)]
mod tests;
