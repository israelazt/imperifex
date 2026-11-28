//! Light and dark: the rule that picks one, the colours of each, and the colours
//! a cell of the grid ends up with.

pub mod cell_colors;
pub mod contrast;
pub mod palette;

use serde::{Deserialize, Serialize};

pub use cell_colors::cell_colors;
pub use palette::{palette, Palette, Rgb};

/// What the user chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeChoice {
    #[default]
    System,
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    pub const ALL: [Self; 2] = [Self::Dark, Self::Light];
}

/// With `System`, what the system says; dark when it is not known.
pub fn resolve_theme(choice: ThemeChoice, system_dark: Option<bool>) -> Theme {
    let dark = match choice {
        ThemeChoice::Dark => true,
        ThemeChoice::Light => false,
        ThemeChoice::System => system_dark.unwrap_or(true),
    };
    if dark {
        Theme::Dark
    } else {
        Theme::Light
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_system_decides_unless_the_user_chose_and_unknown_is_dark() {
        let system = |dark| resolve_theme(ThemeChoice::System, dark);
        assert_eq!(system(Some(true)), Theme::Dark);
        assert_eq!(system(Some(false)), Theme::Light);
        assert_eq!(system(None), Theme::Dark);
        assert_eq!(resolve_theme(ThemeChoice::Dark, Some(false)), Theme::Dark);
        assert_eq!(resolve_theme(ThemeChoice::Light, Some(true)), Theme::Light);
        assert_eq!(ThemeChoice::default(), ThemeChoice::System);
    }
}
