//! How a cell looks. Colours are what the program asked for: which real colour
//! each one is belongs to the theme.

/// A colour asked for by a program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Color {
    /// The theme's own text or background colour.
    #[default]
    Default,
    /// One of the 256 numbered colours; 0 to 15 are the theme's ANSI colours.
    Indexed(u8),
    Rgb(u8, u8, u8),
}

/// A set of on/off attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Attributes(u8);

impl Attributes {
    pub const BOLD: Self = Self(1);
    pub const DIM: Self = Self(1 << 1);
    pub const ITALIC: Self = Self(1 << 2);
    pub const UNDERLINE: Self = Self(1 << 3);
    pub const BLINK: Self = Self(1 << 4);
    pub const INVERSE: Self = Self(1 << 5);
    pub const HIDDEN: Self = Self(1 << 6);
    pub const STRIKE: Self = Self(1 << 7);

    pub const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Whether every attribute of `other` is on.
    pub const fn has(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub foreground: Color,
    pub background: Color,
    pub attributes: Attributes,
}

impl Style {
    /// What an emptied cell keeps: only the background colour, as in xterm.
    pub(super) fn for_emptied(self) -> Self {
        Self {
            background: self.background,
            ..Self::default()
        }
    }
}
