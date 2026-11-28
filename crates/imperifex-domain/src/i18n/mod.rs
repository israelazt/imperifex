//! The languages of the interface: the rule that picks one, its texts, and how
//! it writes counts and numbers. What programs print is never translated.

pub mod number;
pub mod plural;
pub mod strings;

use serde::{Deserialize, Serialize};

pub use strings::{fill, text, Text};

/// What the user chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LanguageChoice {
    #[default]
    System,
    Spanish,
    English,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Spanish,
    English,
}

impl Language {
    pub const ALL: [Self; 2] = [Self::Spanish, Self::English];
}

/// With `System`, a Spanish locale of any country gives Spanish; any other one,
/// or none, gives English.
pub fn resolve_language(choice: LanguageChoice, system_locale: Option<&str>) -> Language {
    match choice {
        LanguageChoice::Spanish => Language::Spanish,
        LanguageChoice::English => Language::English,
        LanguageChoice::System => system_locale.map_or(Language::English, of_locale),
    }
}

/// The language of a tag such as `es-419`: only what is before the country counts.
fn of_locale(tag: &str) -> Language {
    let primary = tag.split(['-', '_']).next().unwrap_or_default();
    if primary.eq_ignore_ascii_case("es") {
        Language::Spanish
    } else {
        Language::English
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_system(locale: Option<&str>) -> Language {
        resolve_language(LanguageChoice::System, locale)
    }

    #[test]
    fn a_spanish_system_of_any_country_gives_spanish() {
        for tag in ["es", "es-MX", "es-419", "es-ES", "ES-mx", "es_AR"] {
            assert_eq!(with_system(Some(tag)), Language::Spanish, "{tag}");
        }
    }

    #[test]
    fn any_other_system_language_or_none_gives_english() {
        for tag in ["en-US", "pt-BR", "et-EE", "est", "eu-ES", ""] {
            assert_eq!(with_system(Some(tag)), Language::English, "{tag}");
        }
        assert_eq!(with_system(None), Language::English);
    }

    #[test]
    fn what_the_user_chose_wins_over_the_system() {
        let spanish = resolve_language(LanguageChoice::Spanish, Some("en-US"));
        let english = resolve_language(LanguageChoice::English, Some("es-MX"));
        assert_eq!(spanish, Language::Spanish);
        assert_eq!(english, Language::English);
        assert_eq!(LanguageChoice::default(), LanguageChoice::System);
    }
}
