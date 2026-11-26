//! Writing an order for the shell of the terminal being measured: Warp's
//! windows run PowerShell (probe 1.3).

/// In single quotes, where PowerShell reads everything as it is and a quote is
/// written twice.
fn quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', "''"))
}

/// The line that runs a program with its arguments.
pub fn powershell_command(exe: &str, args: &[&str]) -> String {
    let mut line = format!("& {}", quoted(exe));
    for arg in args {
        line.push(' ');
        line.push_str(&quoted(arg));
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_with_spaces_stays_one_thing() {
        assert_eq!(
            powershell_command(r"C:\Program Files\b.exe", &["emit", "ascii"]),
            r"& 'C:\Program Files\b.exe' 'emit' 'ascii'"
        );
    }

    #[test]
    fn a_quote_is_written_twice_and_dollars_mean_nothing() {
        assert_eq!(
            powershell_command("it's.exe", &["$env:X"]),
            "& 'it''s.exe' '$env:X'"
        );
    }
}
