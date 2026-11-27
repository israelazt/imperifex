//! `ESC [ … m`: colours and attributes of what is written next.

use crate::grid::{Attributes, Color, Style};
use crate::sequence::Params;

pub(super) fn apply(style: &mut Style, params: Params<'_>) {
    let mut groups = params.groups().peekable();
    // With no parameters it means 0: no style.
    if groups.peek().is_none() {
        *style = Style::default();
    }
    while let Some(group) = groups.next() {
        match group[0] {
            code @ (38 | 48 | 58) => {
                let color = if group.len() > 1 {
                    color_from_parts(&group[1..])
                } else {
                    color_from_next(&mut groups)
                };
                match (code, color) {
                    (38, Some(color)) => style.foreground = color,
                    (48, Some(color)) => style.background = color,
                    // 58 colours the underline: read, so that its numbers are
                    // not taken as codes, and not used.
                    _ => {}
                }
            }
            // `4:0` takes the underline off; any other `4:n` is a kind of it.
            4 if group.get(1) == Some(&0) => off(style, Attributes::UNDERLINE),
            code => one(style, code),
        }
    }
}

fn byte(value: u16) -> Option<u8> {
    u8::try_from(value).ok()
}

fn rgb(parts: &[u16]) -> Option<Color> {
    match parts {
        [r, g, b] => Some(Color::Rgb(byte(*r)?, byte(*g)?, byte(*b)?)),
        _ => None,
    }
}

/// The colon form: `5:n`, `2:r:g:b` or `2:space:r:g:b`.
fn color_from_parts(parts: &[u16]) -> Option<Color> {
    match parts {
        [5, index] => byte(*index).map(Color::Indexed),
        [2, .., r, g, b] => rgb(&[*r, *g, *b]),
        _ => None,
    }
}

/// The semicolon form: the numbers are the parameters that follow.
fn color_from_next<'a>(groups: &mut impl Iterator<Item = &'a [u16]>) -> Option<Color> {
    let mut next = || groups.next().map(|group| group[0]);
    match next()? {
        5 => byte(next()?).map(Color::Indexed),
        2 => {
            let parts = [next()?, next()?, next()?];
            rgb(&parts)
        }
        _ => None,
    }
}

fn on(style: &mut Style, attribute: Attributes) {
    style.attributes = style.attributes.with(attribute);
}

fn off(style: &mut Style, attribute: Attributes) {
    style.attributes = style.attributes.without(attribute);
}

fn one(style: &mut Style, code: u16) {
    match code {
        0 => *style = Style::default(),
        1..=9 | 21 => attribute_on(style, code),
        22..=29 => attribute_off(style, code),
        _ => color(style, code),
    }
}

fn attribute_on(style: &mut Style, code: u16) {
    match code {
        1 => on(style, Attributes::BOLD),
        2 => on(style, Attributes::DIM),
        3 => on(style, Attributes::ITALIC),
        // 21 is a double underline.
        4 | 21 => on(style, Attributes::UNDERLINE),
        5 | 6 => on(style, Attributes::BLINK),
        7 => on(style, Attributes::INVERSE),
        8 => on(style, Attributes::HIDDEN),
        9 => on(style, Attributes::STRIKE),
        _ => {}
    }
}

fn attribute_off(style: &mut Style, code: u16) {
    match code {
        22 => off(style, Attributes::BOLD.with(Attributes::DIM)),
        23 => off(style, Attributes::ITALIC),
        24 => off(style, Attributes::UNDERLINE),
        25 => off(style, Attributes::BLINK),
        27 => off(style, Attributes::INVERSE),
        28 => off(style, Attributes::HIDDEN),
        29 => off(style, Attributes::STRIKE),
        _ => {}
    }
}

/// The 16 numbered colours and the way back to the default ones.
fn color(style: &mut Style, code: u16) {
    let indexed = |base: u16, first: u16| byte(code - base + first).map(Color::Indexed);
    match code {
        30..=37 => style.foreground = indexed(30, 0).unwrap_or_default(),
        39 => style.foreground = Color::Default,
        40..=47 => style.background = indexed(40, 0).unwrap_or_default(),
        49 => style.background = Color::Default,
        90..=97 => style.foreground = indexed(90, 8).unwrap_or_default(),
        100..=107 => style.background = indexed(100, 8).unwrap_or_default(),
        _ => {}
    }
}
