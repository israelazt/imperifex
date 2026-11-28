//! Guards of the architecture (see `ARCHITECTURE.md`). They fail when
//! someone breaks a rule the design depends on, so the rules do not rot.

use std::fs;
use std::path::{Path, PathBuf};

/// The root folder of the workspace.
fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// Where a crate lives: the libraries under `crates/`, the programs under `apps/`.
fn crate_dir(krate: &str) -> PathBuf {
    let app = workspace().join("apps").join(krate);
    if app.is_dir() {
        app
    } else {
        workspace().join("crates").join(krate)
    }
}

/// Every folder with Rust sources of the workspace.
fn source_roots() -> Vec<PathBuf> {
    ["crates", "apps"]
        .iter()
        .map(|d| workspace().join(d))
        .filter(|d| d.is_dir())
        .collect()
}

/// Names of the crates a manifest depends on (all `[*dependencies]` tables).
fn dependencies_of(krate: &str) -> Vec<String> {
    let manifest = crate_dir(krate).join("Cargo.toml");
    let text = fs::read_to_string(&manifest).unwrap();
    let mut in_dependencies = false;
    let mut names = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            let table = line.trim_matches(|c| c == '[' || c == ']');
            in_dependencies = table == "dependencies"
                || table == "build-dependencies"
                || table.ends_with(".dependencies");
        } else if in_dependencies && !line.starts_with('#') {
            let is_entry = line.contains('=') && !line.starts_with('"') && !line.starts_with(' ');
            if let (true, Some(name)) = (is_entry, line.split(['=', '.', ' ']).next()) {
                if !name.is_empty() && !name.starts_with("Win32_") {
                    names.push(name.to_string());
                }
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

/// crate -> every dependency it may have. Dependencies go only downward:
/// domain <- platform, app and ui <- the binary. A new one is added here with
/// a reason.
const ALLOWED: [(&str, &[&str]); 9] = [
    // unicode-width: how many columns a character takes (T-D12). Pure
    // tables kept up to date with Unicode, better than our own.
    (
        "imperifex-domain",
        &["serde", "serde_json", "unicode-width"],
    ),
    // vte: splits the bytes of a shell into sequences (T-D11), behind
    // the `SequenceParser` port. What they mean is decided in domain.
    (
        "imperifex-platform",
        &["imperifex-domain", "vte", "windows"],
    ),
    ("imperifex-app", &["imperifex-domain"]),
    // The interface only knows the domain: what it shows arrives in a
    // `Model` and it answers with a `DrawList` and `Action`s.
    ("imperifex-ui", &["imperifex-domain"]),
    // winit: the window and its events (T-D15), only in the program.
    (
        "imperifex",
        &[
            "imperifex-domain",
            "imperifex-platform",
            "imperifex-app",
            "imperifex-ui",
            "winit",
        ],
    ),
    (
        "imperifex-bench-domain",
        &["imperifex-domain", "serde", "serde_json"],
    ),
    (
        "imperifex-bench-platform",
        &["imperifex-bench-domain", "imperifex-domain", "windows"],
    ),
    (
        "imperifex-bench-app",
        &["imperifex-bench-domain", "imperifex-domain"],
    ),
    (
        "imperifex-bench",
        &[
            "imperifex-domain",
            "imperifex-platform",
            "imperifex-bench-domain",
            "imperifex-bench-platform",
            "imperifex-bench-app",
        ],
    ),
];

#[test]
fn each_crate_depends_only_on_what_the_layers_allow() {
    for (krate, may) in ALLOWED {
        assert!(
            !dependencies_of(krate).is_empty(),
            "no dependencies read for {krate}: the parser is broken"
        );
        for dep in dependencies_of(krate) {
            assert!(
                may.contains(&dep.as_str()),
                "{krate} must not depend on {dep} (allowed: {may:?})"
            );
        }
    }
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Every Rust file of the product: not examples nor integration tests.
fn product_files() -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    for root in source_roots() {
        rust_files(&root, &mut files);
    }
    files
        .into_iter()
        .filter_map(|path| {
            let relative = path.strip_prefix(workspace()).unwrap().to_path_buf();
            let skip = relative
                .components()
                .any(|c| c.as_os_str() == "examples" || c.as_os_str() == "tests");
            let shown = relative.to_string_lossy().replace('\\', "/");
            (!skip).then_some((path, shown))
        })
        .collect()
}

/// The part of a file before its `#[cfg(test)] mod tests`. Other items marked
/// `#[cfg(test)]` (a helper module declared in `lib.rs`) do not end it.
fn before_the_tests_module(text: &str) -> &str {
    let mut from = 0;
    while let Some(found) = text[from..].find("#[cfg(test)]") {
        let at = from + found;
        let after = text[at + "#[cfg(test)]".len()..].trim_start();
        if after.starts_with("mod tests") {
            return &text[..at];
        }
        from = at + 1;
    }
    text
}

/// The code of a file without its line comments.
fn code_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.lines()
        .enumerate()
        .map(|(n, l)| (n + 1, l.split("//").next().unwrap_or("")))
}

#[test]
fn no_source_file_is_too_big() {
    const LIMIT: usize = 400;
    const TESTS_LIMIT: usize = 1000;
    let mut too_big = Vec::new();
    for (path, shown) in product_files() {
        let text = fs::read_to_string(&path).unwrap();
        let is_tests = shown.ends_with("/tests.rs");
        let counted = if is_tests {
            text.as_str()
        } else {
            before_the_tests_module(&text)
        };
        let count = counted.lines().filter(|l| !l.trim().is_empty()).count();
        let limit = if is_tests { TESTS_LIMIT } else { LIMIT };
        if count > limit {
            too_big.push(format!("{shown} has {count} lines (limit {limit})"));
        }
    }
    assert!(
        too_big.is_empty(),
        "split these files:\n{}",
        too_big.join("\n")
    );
}

/// Where the program may print straight to the console, and why. Every other
/// message goes through the log (`log_info!`, `log_warn!`, `log_error!`).
const PRINT_EXCEPTIONS: [(&str, &str); 3] = [
    (
        "apps/imperifex/src/main.rs",
        "the output of an order is the program's output: it goes to the standard output",
    ),
    (
        "apps/imperifex-bench/src/main.rs",
        "the output of an order is the program's output: it goes to the standard output",
    ),
    (
        "crates/imperifex-platform/src/log_console.rs",
        "the log's own sink writes to the standard error stream",
    ),
];

#[test]
fn production_code_logs_instead_of_printing() {
    let mut offenders = Vec::new();
    for (path, shown) in product_files() {
        if shown.ends_with("/tests.rs") || PRINT_EXCEPTIONS.iter().any(|(p, _)| shown.ends_with(p))
        {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        for (number, code) in code_lines(before_the_tests_module(&text)) {
            if ["println!(", "eprintln!(", "print!(", "eprint!("]
                .iter()
                .any(|p| code.contains(p))
            {
                offenders.push(format!("{shown}:{number}"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "use log_info!/log_warn!/log_error! instead of printing (or add an exception with its reason):\n{}",
        offenders.join("\n")
    );
}

/// The crates that may call the system: the terminal's adapters and the bench's.
const PLATFORM_CRATES: [&str; 2] = [
    "crates/imperifex-platform/",
    "crates/imperifex-bench-platform/",
];

#[test]
fn unsafe_lives_only_in_platform() {
    let mut offenders = Vec::new();
    for (path, shown) in product_files() {
        if PLATFORM_CRATES.iter().any(|c| shown.starts_with(c)) {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        for (number, code) in code_lines(&text) {
            if ["unsafe {", "unsafe fn", "unsafe impl", "unsafe extern"]
                .iter()
                .any(|p| code.contains(p))
            {
                offenders.push(format!("{shown}:{number}"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "unsafe code belongs in a platform crate, behind a port:\n{}",
        offenders.join("\n")
    );
}

/// A colour written by hand: built from numbers, one of the constants, or as
/// `#RRGGBB`, `0xRRGGBB` or `0xAARRGGBB`.
fn has_a_loose_colour(code: &str) -> bool {
    let hex_after = |mark: &str, lengths: &[usize]| {
        code.match_indices(mark).any(|(at, _)| {
            let rest = &code[at + mark.len()..];
            let digits = rest
                .chars()
                .take_while(|c| c.is_ascii_hexdigit() || *c == '_')
                .filter(|c| *c != '_')
                .count();
            lengths.contains(&digits)
        })
    };
    code.contains("Rgb(")
        || code.contains("Rgb::")
        || hex_after("#", &[6])
        || hex_after("0x", &[6, 8])
}

/// A text written by hand: any quote that is not part of an empty `""`.
fn has_a_loose_text(code: &str) -> bool {
    code.replace("\"\"", "").contains('"')
}

/// The lines of the interface's own code, tests aside, where `wrong` is found.
/// Whole lines are looked at, comments removed only when they start the line: a
/// `//` inside a text must not hide it.
fn in_the_interface(wrong: fn(&str) -> bool) -> Vec<String> {
    let interface = "crates/imperifex-ui/";
    let files: Vec<_> = product_files()
        .into_iter()
        .filter(|(_, shown)| shown.starts_with(interface) && !shown.ends_with("/tests.rs"))
        .collect();
    assert!(!files.is_empty(), "no file of the interface was read");
    let mut offenders = Vec::new();
    for (path, shown) in files {
        let text = fs::read_to_string(&path).unwrap();
        for (at, line) in before_the_tests_module(&text).lines().enumerate() {
            if !line.trim_start().starts_with("//") && wrong(line) {
                offenders.push(format!("{shown}:{}", at + 1));
            }
        }
    }
    offenders
}

#[test]
fn ui_has_no_loose_colours() {
    let offenders = in_the_interface(has_a_loose_colour);
    assert!(
        offenders.is_empty(),
        "the interface takes its colours from the model's palette:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn ui_has_no_loose_visible_text() {
    let offenders = in_the_interface(has_a_loose_text);
    assert!(
        offenders.is_empty(),
        "the interface takes its texts from `Text` or from the model:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn the_guards_of_the_interface_know_what_they_look_for() {
    for code in [
        "let c = Rgb(1, 2, 3);",
        "let c = Rgb::BLACK;",
        "fill(#A0b1C2)",
        "hex(0x0F0F10)",
        "hex(0xFF_0F_0F_10)",
    ] {
        assert!(has_a_loose_colour(code), "{code}");
    }
    for code in ["#[derive(Debug)]", "let mask = 0xFF;", "palette.surface"] {
        assert!(!has_a_loose_colour(code), "{code}");
    }
    assert!(has_a_loose_text(r#"label("Close")"#));
    assert!(has_a_loose_text(r#"open("https://x")"#));
    assert!(has_a_loose_text(r#"join("", " ")"#));
    assert!(!has_a_loose_text(r#"let empty = "";"#));
    assert!(!has_a_loose_text("text(model.language, Text::Close)"));
}

#[test]
fn only_the_tests_module_ends_the_production_code() {
    let file = "fn a() {}\n#[cfg(test)]\nmod testing;\nfn b() {}\n#[cfg(test)]\nmod tests {\nfn c() {}\n}\n";
    let production = before_the_tests_module(file);
    assert!(production.contains("fn b()"));
    assert!(!production.contains("fn c()"));
    assert_eq!(before_the_tests_module("fn only() {}"), "fn only() {}");
}
