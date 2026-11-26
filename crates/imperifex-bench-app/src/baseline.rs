//! The baseline document, made again from the reports that were kept: every
//! number in it comes from one of them.

use std::path::{Path, PathBuf};

use imperifex_bench_domain::markdown::baseline_markdown;
use imperifex_bench_domain::ports::FileStore;
use imperifex_bench_domain::report::Report;

const REPORT_FILE: &str = "report.json";

fn report_in(files: &dyn FileStore, folder: &Path) -> Result<Report, String> {
    let file = folder.join(REPORT_FILE);
    let bytes = files
        .read(&file)?
        .ok_or_else(|| format!("no hay {}", file.display()))?;
    let json = String::from_utf8(bytes).map_err(|_| format!("{} no es texto", file.display()))?;
    Report::from_json(&json).map_err(|error| format!("{}: {error}", file.display()))
}

/// Writes in `out` the tables of the reports of `folders`, numbered in the
/// order given.
pub fn write_baseline(
    files: &dyn FileStore,
    folders: &[PathBuf],
    out: &Path,
) -> Result<(), String> {
    let reports: Vec<Report> = folders
        .iter()
        .map(|folder| report_in(files, folder))
        .collect::<Result<_, _>>()?;
    files.write(out, baseline_markdown(&reports).as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::FakeFiles;

    fn with_report(files: &FakeFiles, folder: &str, date: &str) {
        let report = Report {
            date: date.into(),
            ..Default::default()
        };
        let file = Path::new(folder).join(REPORT_FILE);
        files.write(&file, report.to_json().as_bytes()).unwrap();
    }

    #[test]
    fn the_reports_become_runs_in_the_order_given() {
        let files = FakeFiles::default();
        with_report(&files, "second", "2026-11-26");
        with_report(&files, "first", "2026-11-25");
        let folders = [PathBuf::from("first"), PathBuf::from("second")];
        write_baseline(&files, &folders, Path::new("B.md")).unwrap();
        let text = files.read(Path::new("B.md")).unwrap().unwrap();
        let text = String::from_utf8(text).unwrap();
        assert!(text.contains("| 1 | 2026-11-25 |") && text.contains("| 2 | 2026-11-26 |"));
    }

    #[test]
    fn a_folder_without_a_report_is_an_error_and_nothing_is_written() {
        let files = FakeFiles::default();
        let folders = [PathBuf::from("empty")];
        let error = write_baseline(&files, &folders, Path::new("B.md")).unwrap_err();
        assert!(
            error.contains("empty") && error.contains(REPORT_FILE),
            "{error}"
        );
        assert_eq!(files.read(Path::new("B.md")), Ok(None));
    }

    #[test]
    fn what_is_not_a_report_says_which_file_it_was() {
        let files = FakeFiles::default();
        let file = Path::new("bad").join(REPORT_FILE);
        files.write(&file, b"{}").unwrap();
        let error = write_baseline(&files, &["bad".into()], Path::new("B.md")).unwrap_err();
        assert!(error.contains("bad"), "{error}");
    }
}
