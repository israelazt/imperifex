//! The colours of each theme. The only place where a colour is written.

use super::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub const BLACK: Self = Self(0, 0, 0);
    pub const WHITE: Self = Self(255, 255, 255);

    /// From `0xRRGGBB`, as colours are usually written.
    const fn hex(value: u32) -> Self {
        let [_, r, g, b] = value.to_be_bytes();
        Self(r, g, b)
    }
}

/// What each colour is for. A state is never said with its colour alone.
#[derive(Debug, PartialEq, Eq)]
pub struct Palette {
    /// The window.
    pub background: Rgb,
    /// The output.
    pub surface: Rgb,
    /// The active tab and cards.
    pub raised: Rgb,
    pub text: Rgb,
    /// Secondary text.
    pub muted: Rgb,
    /// The prompt, the cursor and the selection.
    pub accent: Rgb,
    /// It went well, or it is working.
    pub ok: Rgb,
    pub warning: Rgb,
    /// It failed.
    pub error: Rgb,
    /// An agent waits for the user.
    pub waiting: Rgb,
    /// The 16 colours a program asks for by number.
    pub ansi: [Rgb; 16],
}

const DARK: Palette = Palette {
    background: Rgb::hex(0x131315),
    surface: Rgb::hex(0x0F0F10),
    raised: Rgb::hex(0x1C1C1F),
    text: Rgb::hex(0xE4E4E7),
    muted: Rgb::hex(0x9A9AA3),
    accent: Rgb::hex(0x4CD7F6),
    ok: Rgb::hex(0x4ADE9B),
    warning: Rgb::hex(0xF5C451),
    error: Rgb::hex(0xFF7B72),
    waiting: Rgb::hex(0xC4A7FF),
    ansi: ansi([
        0x1C1C1F, 0xFF7B72, 0x4ADE9B, 0xF5C451, 0x6CA8FF, 0xC4A7FF, 0x4CD7F6, 0xC8C8CE, 0x6B6B75,
        0xFFA198, 0x7EEDB8, 0xFFD97A, 0x9CC5FF, 0xDCCBFF, 0x8BE6FA, 0xF4F4F5,
    ]),
};

const LIGHT: Palette = Palette {
    background: Rgb::hex(0xF4F4F5),
    surface: Rgb::hex(0xFFFFFF),
    raised: Rgb::hex(0xE9E9EC),
    text: Rgb::hex(0x18181B),
    muted: Rgb::hex(0x55555E),
    accent: Rgb::hex(0x006781),
    ok: Rgb::hex(0x0F6B42),
    warning: Rgb::hex(0x7A5200),
    error: Rgb::hex(0xB3261E),
    waiting: Rgb::hex(0x6A3FC0),
    ansi: ansi([
        0x18181B, 0xB3261E, 0x0F6B42, 0x7A5200, 0x1F5FBF, 0x6A3FC0, 0x006781, 0xE4E4E7, 0x55555E,
        0x9E1F18, 0x0B5C38, 0x664400, 0x1A50A3, 0x5A33A8, 0x00576E, 0xFFFFFF,
    ]),
};

const fn ansi(values: [u32; 16]) -> [Rgb; 16] {
    let mut colours = [Rgb::BLACK; 16];
    let mut at = 0;
    while at < values.len() {
        colours[at] = Rgb::hex(values[at]);
        at += 1;
    }
    colours
}

pub fn palette(theme: Theme) -> &'static Palette {
    match theme {
        Theme::Dark => &DARK,
        Theme::Light => &LIGHT,
    }
}
