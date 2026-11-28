//! The commands run in the editor, in a file per profile.

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use imperifex_domain::ports::CommandHistory;

pub struct FileCommandHistory {
    folder: PathBuf,
}

impl FileCommandHistory {
    /// `folder` is created when something is first written in it.
    pub fn new(folder: PathBuf) -> Self {
        Self { folder }
    }

    /// `%APPDATA%\Imperifex\history`.
    pub fn in_app_data() -> Result<Self, String> {
        let app_data = std::env::var_os("APPDATA").ok_or("no hay %APPDATA%")?;
        let folder = Path::new(&app_data).join("Imperifex").join("history");
        Ok(Self::new(folder))
    }

    /// The file of a profile. A name that is not plain letters, digits, `-` or
    /// `_` has the rest replaced: it comes from the settings.
    fn file(&self, profile: &str) -> PathBuf {
        let name: String = profile
            .chars()
            .map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
                _ => '_',
            })
            .collect();
        self.folder.join(format!("{name}.txt"))
    }

    fn make_folder(&self) -> Result<(), String> {
        fs::create_dir_all(&self.folder).map_err(|e| failed("crear", &self.folder, &e))
    }
}

fn failed(doing: &str, path: &Path, error: &std::io::Error) -> String {
    format!("no se pudo {doing} {}: {error}", path.display())
}

impl CommandHistory for FileCommandHistory {
    fn load(&self, profile: &str) -> Result<Option<String>, String> {
        let file = self.file(profile);
        match fs::read(&file) {
            Ok(bytes) => Ok(Some(String::from_utf8_lossy(&bytes).into_owned())),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(failed("leer", &file, &error)),
        }
    }

    fn append(&self, profile: &str, line: &str) -> Result<(), String> {
        self.make_folder()?;
        let file = self.file(profile);
        let mut open = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file)
            .map_err(|e| failed("abrir", &file, &e))?;
        open.write_all(line.as_bytes())
            .map_err(|e| failed("escribir", &file, &e))
    }

    fn replace(&self, profile: &str, text: &str) -> Result<(), String> {
        self.make_folder()?;
        let file = self.file(profile);
        let temporary = file.with_extension("tmp");
        fs::write(&temporary, text).map_err(|e| failed("escribir", &temporary, &e))?;
        fs::rename(&temporary, &file).map_err(|e| failed("reemplazar", &file, &e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A folder of the test's own, removed at the end.
    struct Folder(PathBuf);

    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn each_profile_has_its_file_which_grows_and_can_be_written_again() {
        let name = format!("imperifex-history-{}", std::process::id());
        let folder = Folder(std::env::temp_dir().join(name));
        let history = FileCommandHistory::new(folder.0.join("history"));
        assert_eq!(history.load("powershell"), Ok(None));
        history.append("powershell", "one\n").expect("added");
        history.append("powershell", "two\n").expect("added");
        history.append("pwsh", "other\n").expect("added");
        let found = history.load("powershell").expect("read");
        assert_eq!(found.as_deref(), Some("one\ntwo\n"));
        history.replace("powershell", "two\n").expect("written");
        let found = history.load("powershell").expect("read");
        assert_eq!(found.as_deref(), Some("two\n"));
        // A name with a path in it does not leave the folder.
        history.append(r"..\..\evil", "x\n").expect("added");
        assert!(folder.0.join("history").join("______evil.txt").exists());
    }
}
