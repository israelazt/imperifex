//! The files of the bench, on disk.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use imperifex_bench_domain::ports::FileStore;

/// Reads and writes real files.
#[derive(Debug, Default)]
pub struct DiskFiles;

fn failed<'a>(what: &'a str, path: &'a Path) -> impl Fn(std::io::Error) -> String + 'a {
    move |error| format!("no se pudo {what} {}: {error}", path.display())
}

impl FileStore for DiskFiles {
    /// Writes a `.tmp` file beside the target and renames it over it.
    fn write(&self, path: &Path, bytes: &[u8]) -> Result<(), String> {
        if let Some(folder) = path.parent() {
            fs::create_dir_all(folder).map_err(failed("crear la carpeta de", path))?;
        }
        let mut name = path.as_os_str().to_owned();
        name.push(".tmp");
        let temporary = PathBuf::from(name);
        fs::write(&temporary, bytes).map_err(failed("escribir", &temporary))?;
        fs::rename(&temporary, path).map_err(|error| {
            // Nothing half done is left behind; the first error is the one told.
            let _ = fs::remove_file(&temporary);
            failed("dejar en su sitio", path)(error)
        })
    }

    fn read(&self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        match fs::read(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(failed("leer", path)(error)),
        }
    }

    fn temp_dir(&self) -> Result<PathBuf, String> {
        Ok(std::env::temp_dir().join("imperifex-bench"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A folder of its own for each test, removed when it ends, pass or fail.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Self {
            let folder = format!("imperifex-bench-test-{}-{name}", std::process::id());
            Self(std::env::temp_dir().join(folder))
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn what_is_written_reads_back_and_replaces_what_was_there() {
        let scratch = Scratch::new("replace");
        let path = scratch.0.join("deeper").join("a.json");
        DiskFiles.write(&path, b"first").unwrap();
        DiskFiles.write(&path, b"second").unwrap();
        assert_eq!(DiskFiles.read(&path), Ok(Some(b"second".to_vec())));
        let left: Vec<_> = fs::read_dir(path.parent().unwrap()).unwrap().collect();
        assert_eq!(left.len(), 1, "no temporary file stays");
    }

    #[test]
    fn a_file_that_is_not_there_is_nothing_and_not_an_error() {
        let scratch = Scratch::new("missing");
        assert_eq!(DiskFiles.read(&scratch.0.join("no.json")), Ok(None));
    }

    #[test]
    fn a_write_that_fails_says_where_and_leaves_no_temporary_file() {
        let scratch = Scratch::new("blocked");
        let taken = scratch.0.join("folder");
        fs::create_dir_all(taken.join("inside")).unwrap();
        let error = DiskFiles.write(&taken, b"x").unwrap_err();
        assert!(error.contains("folder"), "{error}");
        assert!(!scratch.0.join("folder.tmp").exists());
    }
}
