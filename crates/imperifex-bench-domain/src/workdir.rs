//! Where the measures that open windows are run from. A new window of Warp
//! takes the folder and the Git state of the active tab (probe 1.4), so runs
//! only compare when they all start from the same folder, outside any
//! repository (D15).

use std::path::{Path, PathBuf};

/// The first folder, from `dir` upwards, that holds a `.git`.
pub fn repo_around(dir: &Path, exists: &dyn Fn(&Path) -> bool) -> Option<PathBuf> {
    dir.ancestors()
        .find(|folder| exists(&folder.join(".git")))
        .map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(dir: &str) -> Option<PathBuf> {
        let git = Path::new("work").join("project").join(".git");
        repo_around(&PathBuf::from_iter(dir.split('/')), &|path| path == git)
    }

    #[test]
    fn deep_inside_a_repository_its_root_is_found() {
        assert_eq!(
            found("work/project/crates/one"),
            Some(PathBuf::from_iter(["work", "project"]))
        );
    }

    #[test]
    fn the_root_of_a_repository_is_inside_it() {
        assert_eq!(
            found("work/project"),
            Some(PathBuf::from_iter(["work", "project"]))
        );
    }

    #[test]
    fn outside_there_is_none() {
        assert_eq!(found("work/other"), None);
        assert_eq!(found("work"), None);
    }
}
