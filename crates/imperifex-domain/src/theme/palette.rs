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
    /// Lines that only separate: nothing is written in it.
    pub line: Rgb,
    /// Under the button that closes while the mouse is on it, as in Windows,
    /// and the colour of its icon then.
    pub close_hover: Rgb,
    pub on_close_hover: Rgb,
    /// The 16 colours a program asks for by number.
    pub ansi: [Rgb; 16],
}

const DARK: Palette = Palette {
    background: Rgb::hex(0x202020),
    surface: Rgb::hex(0x1A1A1A),
    raised: Rgb::hex(0x2D2D2D),
    text: Rgb::hex(0xE4E4E4),
    muted: Rgb::hex(0xA6A6A6),
    accent: Rgb::hex(0x6EE7C0),
    ok: Rgb::hex(0xA5D66F),
    warning: Rgb::hex(0xF5C451),
    error: Rgb::hex(0xFF7B72),
    waiting: Rgb::hex(0xC4A7FF),
    line: Rgb::hex(0x383838),
    close_hover: Rgb::hex(0xC42B1C),
    on_close_hover: Rgb::WHITE,
    ansi: ansi([
        0x5C5C5C, 0xFF7B72, 0xA5D66F, 0xF5C451, 0x7AA7FF, 0xE08BF0, 0x4CD7F6, 0xCACACA, 0x8C8C8C,
        0xFFA198, 0xC2E89A, 0xFFD97A, 0xA5C3FF, 0xEDB1F7, 0x8BE6FA, 0xF4F4F5,
    ]),
};

/// Its "white" ones (7 and 15) are dark greys, so that what a program writes in
/// white is read on white (T-D20).
const LIGHT: Palette = Palette {
    background: Rgb::hex(0xF4F4F5),
    surface: Rgb::hex(0xFFFFFF),
    raised: Rgb::hex(0xE9E9EC),
    text: Rgb::hex(0x18181B),
    muted: Rgb::hex(0x55555E),
    accent: Rgb::hex(0x006B54),
    ok: Rgb::hex(0x3D6B00),
    warning: Rgb::hex(0x7A5200),
    error: Rgb::hex(0xB3261E),
    waiting: Rgb::hex(0x6A3FC0),
    line: Rgb::hex(0xDEDEE2),
    close_hover: Rgb::hex(0xC42B1C),
    on_close_hover: Rgb::WHITE,
    ansi: ansi([
        0x18181B, 0xB3261E, 0x3D6B00, 0x7A5200, 0x1F4FBF, 0x8A2E9E, 0x006781, 0x55555E, 0x6B6B74,
        0xC2352A, 0x437600, 0x8A5D00, 0x2B5FD9, 0x9E3BB3, 0x00708C, 0x3F3F46,
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
