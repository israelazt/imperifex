//! The settings file on disk, written through a temporary one so that it is
//! never left half written.

use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use imperifex_domain::ports::SettingsStore;

const FILE: &str = "settings.json";

/// More copies of damaged files than this are not kept.
const COPIES: u32 = 100;

pub struct FileSettingsStore {
    folder: PathBuf,
}

impl FileSettingsStore {
    /// `folder` is created when something is first written in it.
    pub fn new(folder: PathBuf) -> Self {
        Self { folder }
    }

    /// `%APPDATA%\Imperifex`.
    pub fn in_app_data() -> Result<Self, String> {
        let app_data = std::env::var_os("APPDATA").ok_or("no hay %APPDATA%")?;
        Ok(Self::new(Path::new(&app_data).join("Imperifex")))
    }

    fn file(&self) -> PathBuf {
        self.folder.join(FILE)
    }
}

impl SettingsStore for FileSettingsStore {
    fn load(&self) -> Result<Option<String>, String> {
        let found = read_if_there(&self.file())?;
        Ok(found.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()))
    }

    fn save(&self, text: &str) -> Result<(), String> {
        fs::create_dir_all(&self.folder).map_err(|e| failed("crear", &self.folder, &e))?;
        let temporary = self.folder.join("settings.json.tmp");
        write_to_disk(&temporary, text.as_bytes())
            .map_err(|e| failed("escribir", &temporary, &e))?;
        // On Windows this replaces the file in one step.
        let file = self.file();
        fs::rename(&temporary, &file).map_err(|e| failed("reemplazar", &file, &e))
    }

    fn keep_damaged(&self, text: &str) -> Result<String, String> {
        // The bytes of the file, if `text` is what it says: those that were not
        // text are only kept this way.
        let bytes = read_if_there(&self.file())?
            .filter(|bytes| String::from_utf8_lossy(bytes) == text)
            .unwrap_or_else(|| text.as_bytes().to_vec());
        for number in 1..=COPIES {
            let copy = self.folder.join(copy_name(number));
            match read_if_there(&copy)? {
                // The same file as an earlier start: one copy is enough.
                Some(kept) if kept == bytes => return Ok(shown(&copy)),
                Some(_) => {}
                None => return keep_new(&copy, &bytes),
            }
        }
        Err(format!("ya hay {COPIES} copias de ajustes dañados"))
    }
}

fn copy_name(number: u32) -> String {
    match number {
        1 => "settings.damaged.json".into(),
        _ => format!("settings.damaged-{number}.json"),
    }
}

fn read_if_there(file: &Path) -> Result<Option<Vec<u8>>, String> {
    match fs::read(file) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(failed("leer", file, &error)),
    }
}

/// Writes a file that must not exist yet.
fn keep_new(copy: &Path, bytes: &[u8]) -> Result<String, String> {
    fs::File::create_new(copy)
        .and_then(|mut file| file.write_all(bytes).and_then(|()| file.sync_all()))
        .map_err(|e| failed("copiar a", copy, &e))?;
    Ok(shown(copy))
}

fn write_to_disk(file: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = fs::File::create(file)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn shown(path: &Path) -> String {
    path.display().to_string()
}

fn failed(what: &str, path: &Path, error: &std::io::Error) -> String {
    format!("no se pudo {what} {}: {error}", shown(path))
}

#[cfg(test)]
mod tests;
