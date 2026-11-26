//! The folder the windows of a measure open from (D15).

use std::path::PathBuf;

use imperifex_bench_domain::ports::FileStore;
use imperifex_bench_domain::workdir::repo_around;

/// Inside the bench's folder for passing files.
const HOME: &str = "home";

/// The folder to launch windows from, made if it is not there. The new window
/// opens in it, wherever the bench itself was started; it is an error if it is
/// inside a repository, as the window would show its Git state.
pub fn measure_home(files: &dyn FileStore) -> Result<PathBuf, String> {
    let home = files.temp_dir()?.join(HOME);
    files.make_dir(&home)?;
    if let Some(repo) = repo_around(&home, &|path| files.exists(path)) {
        return Err(format!(
            "la carpeta de las medidas está dentro de un repositorio ({}): cambia la carpeta temporal del sistema",
            repo.display()
        ));
    }
    Ok(home)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::testing::FakeFiles;

    fn files_with_a_repository(at: &str) -> FakeFiles {
        let files = FakeFiles::default();
        files.make_dir(&Path::new(at).join(".git")).unwrap();
        files
    }

    #[test]
    fn outside_a_repository_the_home_is_made_and_given() {
        let files = files_with_a_repository("project");
        let home = measure_home(&files).unwrap();
        assert_eq!(home, Path::new("temp").join(HOME));
        assert!(files.exists(&home));
    }

    #[test]
    fn a_home_inside_a_repository_is_refused() {
        let files = files_with_a_repository("temp");
        let error = measure_home(&files).unwrap_err();
        assert!(error.contains("carpeta temporal"), "{error}");
    }

    #[test]
    fn a_folder_that_cannot_be_made_is_an_error() {
        let files = FakeFiles {
            failure: Some("denegado".into()),
            ..Default::default()
        };
        assert_eq!(measure_home(&files), Err("denegado".into()));
    }
}
