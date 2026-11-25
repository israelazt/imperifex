//! Taking personal paths out of a recording before it goes in the repository,
//! which is public (safety rule S5).

/// Where every crate of a scrubbed recording says it is.
const NEUTRAL: &str = r"C:\src\imperifex";

/// Replaces the absolute Windows path in parentheses that follows `Compiling`
/// on a line. Everything else is left as it is.
pub fn scrub_paths(text: &str) -> String {
    text.split_inclusive('\n').map(scrub_line).collect()
}

fn scrub_line(line: &str) -> String {
    let found = line.find("Compiling").and_then(|compiling| {
        let open = compiling + line[compiling..].find('(')? + 1;
        // The last one: a folder such as `Program Files (x86)` has its own.
        let close = line.rfind(')')?;
        let path = line.get(open..close)?;
        is_windows_absolute(path).then_some((open, close))
    });
    match found {
        Some((open, close)) => format!("{}{NEUTRAL}{}", &line[..open], &line[close..]),
        None => line.to_string(),
    }
}

/// Starts with a drive, as in `C:\`.
fn is_windows_absolute(path: &str) -> bool {
    matches!(
        path.as_bytes(),
        [drive, b':', b'\\' | b'/', ..] if drive.is_ascii_alphabetic()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const COLORED: &str = "\x1b[92m   Compiling\x1b[0m thing v0.1.0 ";

    fn compiling_in(folder: &str) -> String {
        format!("{COLORED}({folder})\n")
    }

    #[test]
    fn the_folder_of_a_crate_being_compiled_is_replaced() {
        let text = compiling_in(r"D:\home\someone\work\thing");
        assert_eq!(scrub_paths(&text), compiling_in(NEUTRAL));
    }

    #[test]
    fn a_folder_with_parentheses_is_replaced_whole() {
        let text = compiling_in("C:/Program Files (x86)/thing");
        assert_eq!(scrub_paths(&text), compiling_in(NEUTRAL));
    }

    #[test]
    fn every_line_is_looked_at_and_the_rest_is_left_alone() {
        let text = "Compiling a (E:/one)\nFinished (E:/kept)\nCompiling b v1 (proc-macro)\n";
        assert_eq!(
            scrub_paths(text),
            format!("Compiling a ({NEUTRAL})\nFinished (E:/kept)\nCompiling b v1 (proc-macro)\n")
        );
    }
}
