//! The folder the windows of a measure open from (D15).

use std::path::{Path, PathBuf};

use imperifex_bench_domain::ports::FileStore;
use imperifex_bench_domain::workdir::repo_around;

/// Inside the bench's folder for passing files.
const HOME: &str = "home";

/// The folder to launch windows from, made if it is not there. It refuses to go
/// on if the bench itself was started inside a repository, as the new window
/// would copy the folder and the Git state of the tab it came from.
pub fn measure_home(files: &dyn FileStore, current: &Path) -> Result<PathBuf, String> {
    let home = files.temp_dir()?.join(HOME);
    files.make_dir(&home)?;
    let in_a_repository = |dir: &Path| repo_around(dir, &|path| files.exists(path));
    if in_a_repository(current).is_some() {
        return Err(format!(
            "lánzalo desde una carpeta sin Git: cd \"{}\"; la ventana nueva copia la carpeta y el Git de la pestaña activa y las tandas no se podrían comparar",
            home.display()
        ));
    }
    if let Some(repo) = in_a_repository(&home) {
        return Err(format!(
            "la carpeta de las medidas está dentro de un repositorio ({}): cambia la carpeta temporal del sistema",
            repo.display()
        ));
    }
    Ok(home)
}

#[cfg(test)]
mod tests {
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
        let home = measure_home(&files, Path::new("elsewhere")).unwrap();
        assert_eq!(home, Path::new("temp").join(HOME));
        assert!(files.exists(&home));
    }

    #[test]
    fn inside_a_repository_it_refuses_and_says_where_to_go() {
        let files = files_with_a_repository("project");
        let current = Path::new("project").join("src");
        let error = measure_home(&files, &current).unwrap_err();
        assert!(error.contains("cd \"") && error.contains(HOME), "{error}");
    }

    #[test]
    fn a_home_inside_a_repository_is_refused_too() {
        let files = files_with_a_repository("temp");
        let error = measure_home(&files, Path::new("elsewhere")).unwrap_err();
        assert!(error.contains("carpeta temporal"), "{error}");
    }

    #[test]
    fn a_folder_that_cannot_be_made_is_an_error() {
        let files = FakeFiles {
            failure: Some("denegado".into()),
            ..Default::default()
        };
        assert_eq!(
            measure_home(&files, Path::new("elsewhere")),
            Err("denegado".into())
        );
    }
}
