//! The settings while the program runs: read once, and written only when they
//! change. A damaged file is copied before anything can write over it.

use imperifex_domain::i18n::{text, Language, Text};
use imperifex_domain::ports::SettingsStore;
use imperifex_domain::settings::Settings;
use imperifex_domain::{log_info, log_warn};

/// What the user must be told about the settings file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notice {
    /// Part of it could not be read; the file as it was is in `kept`.
    Damaged { kept: String },
    /// It could not be read, or not copied: it is left alone.
    Locked { reason: String },
}

impl Notice {
    /// What to tell the user, with where the copy is or why nothing is written.
    pub fn text(&self, language: Language) -> String {
        let (key, detail) = match self {
            Self::Damaged { kept } => (Text::SettingsDamaged, kept),
            Self::Locked { reason } => (Text::SettingsLocked, reason),
        };
        format!("{} ({detail})", text(language, key))
    }
}

pub struct SettingsService {
    store: Box<dyn SettingsStore>,
    /// What was read or last written.
    current: Settings,
    /// Why nothing may be written, when something could be lost.
    locked: Option<String>,
}

impl SettingsService {
    /// Never fails: what cannot be read has its default. With no file nothing is
    /// written.
    pub fn load(store: Box<dyn SettingsStore>) -> (Self, Option<Notice>) {
        let (current, notice) = read(store.as_ref());
        let locked = match &notice {
            Some(Notice::Locked { reason }) => Some(reason.clone()),
            _ => None,
        };
        let service = Self {
            store,
            current,
            locked,
        };
        (service, notice)
    }

    pub fn settings(&self) -> &Settings {
        &self.current
    }

    /// Writes `settings`, only if they differ from what there is.
    pub fn save(&mut self, settings: Settings) -> Result<(), String> {
        if settings == self.current {
            return Ok(());
        }
        if let Some(reason) = &self.locked {
            return Err(format!("no se escribe sobre los ajustes: {reason}"));
        }
        self.store.save(&settings.write())?;
        self.current = settings;
        Ok(())
    }
}

fn read(store: &dyn SettingsStore) -> (Settings, Option<Notice>) {
    let text = match store.load() {
        Ok(Some(text)) => text,
        Ok(None) => return (Settings::default(), None),
        Err(reason) => return (Settings::default(), Some(locked(reason))),
    };
    let found = Settings::read(&text);
    if !found.damaged {
        return (found.settings, None);
    }
    let notice = match store.keep_damaged(&text) {
        Ok(kept) => {
            log_info!("damaged settings file; a copy is in {kept}");
            Notice::Damaged { kept }
        }
        Err(reason) => locked(reason),
    };
    (found.settings, Some(notice))
}

fn locked(reason: String) -> Notice {
    log_warn!("the settings file is left alone: {reason}");
    Notice::Locked { reason }
}

#[cfg(test)]
mod tests;
