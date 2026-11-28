//! What the user can choose, its values by default, and reading it back from a
//! file that may be old, damaged or edited by hand.

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::i18n::LanguageChoice;
use crate::theme::ThemeChoice;

/// The version of the file written today.
const VERSION: u32 = 1;

/// The sizes a font can have, in points.
const FONT_SIZES: std::ops::RangeInclusive<f64> = 6.0..=72.0;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FontSettings {
    pub family: String,
    /// In points.
    pub size: f32,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            family: "Cascadia Mono".into(),
            size: 12.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Settings {
    pub version: u32,
    pub theme: ThemeChoice,
    pub language: LanguageChoice,
    pub font: FontSettings,
    /// The shell a new tab opens; none chosen: the best one installed.
    pub shell: Option<String>,
    /// Whether text that would not be seen is made visible (T-D17).
    pub adjust_contrast: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: VERSION,
            theme: ThemeChoice::default(),
            language: LanguageChoice::default(),
            font: FontSettings::default(),
            shell: None,
            adjust_contrast: true,
        }
    }
}

/// What reading a file gave.
#[derive(Debug, Clone, PartialEq)]
pub struct Read {
    pub settings: Settings,
    /// Something in it could not be read and has its default instead.
    pub damaged: bool,
}

impl Settings {
    /// Never fails: each field is read on its own, so one that is wrong takes its
    /// default and the rest are kept. A missing field takes its default and an
    /// unknown one is ignored, and neither is damage.
    pub fn read(text: &str) -> Read {
        // Some editors start the file with a byte order mark.
        let text = text.strip_prefix('\u{FEFF}').unwrap_or(text);
        // The replacement character stands for bytes that were not text.
        let mut damaged = text.contains('\u{FFFD}');
        let mut settings = Self::default();
        match serde_json::from_str::<Value>(text) {
            Ok(Value::Object(fields)) => {
                let mut fields = Fields {
                    map: &fields,
                    damaged: &mut damaged,
                };
                settings.take(&mut fields);
            }
            _ => damaged = true,
        }
        Read { settings, damaged }
    }

    fn take(&mut self, fields: &mut Fields<'_>) {
        fields.read("version", &mut self.version, |value| {
            value.as_u64().map(|_| VERSION)
        });
        fields.read("theme", &mut self.theme, named);
        fields.read("language", &mut self.language, named);
        fields.read("shell", &mut self.shell, |value| match value {
            Value::Null => Some(None),
            other => filled_text(other).map(Some),
        });
        fields.read("adjust_contrast", &mut self.adjust_contrast, Value::as_bool);
        fields.read_object("font", |font| {
            font.read("family", &mut self.font.family, filled_text);
            font.read("size", &mut self.font.size, font_size);
        });
    }

    /// The text of the file, indented so that it can be edited by hand.
    pub fn write(&self) -> String {
        let mut text = serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into());
        text.push('\n');
        text
    }
}

/// A choice written as one of its names.
fn named<T: DeserializeOwned>(value: &Value) -> Option<T> {
    T::deserialize(value).ok()
}

/// The fields of one object of the file, noting whether any was wrong.
struct Fields<'a> {
    map: &'a Map<String, Value>,
    damaged: &'a mut bool,
}

impl Fields<'_> {
    /// Puts the field `name` in `into`, if it is there and `parse` accepts it.
    fn read<T>(&mut self, name: &str, into: &mut T, parse: impl FnOnce(&Value) -> Option<T>) {
        let Some(value) = self.map.get(name) else {
            return;
        };
        match parse(value) {
            Some(parsed) => *into = parsed,
            None => *self.damaged = true,
        }
    }

    fn read_object(&mut self, name: &str, read: impl FnOnce(&mut Fields<'_>)) {
        match self.map.get(name) {
            None => {}
            Some(Value::Object(map)) => read(&mut Fields {
                map,
                damaged: self.damaged,
            }),
            Some(_) => *self.damaged = true,
        }
    }
}

fn filled_text(value: &Value) -> Option<String> {
    let text = value.as_str()?;
    (!text.trim().is_empty()).then(|| text.to_string())
}

fn font_size(value: &Value) -> Option<f32> {
    let size = value.as_f64().filter(|size| FONT_SIZES.contains(size))?;
    // Between 6 and 72, which an `f32` holds.
    Some(size as f32)
}

#[cfg(test)]
mod tests;
