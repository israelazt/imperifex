//! What is in a folder of the disk.

use std::fs;

use imperifex_domain::ports::Folders;

/// More names than this are not listed: `Tab` is not for going through them.
const MOST: usize = 5000;

pub struct DiskFolders;

impl Folders for DiskFolders {
    fn entries(&self, path: &str) -> Result<Vec<(String, bool)>, String> {
        let listed = fs::read_dir(path).map_err(|e| format!("no se pudo leer {path}: {e}"))?;
        Ok(listed
            .filter_map(Result::ok)
            .take(MOST)
            .map(|entry| {
                let is_folder = entry.file_type().is_ok_and(|kind| kind.is_dir());
                (entry.file_name().to_string_lossy().into_owned(), is_folder)
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sources_of_this_crate_are_listed_and_a_missing_folder_is_an_error() {
        let source = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
        let found = DiskFolders.entries(source).expect("listed");
        assert!(found.contains(&("folders.rs".into(), false)));
        assert!(found.contains(&("pty".into(), true)));
        assert!(DiskFolders.entries("Z:/no/such/folder/8d2f").is_err());
    }
}
