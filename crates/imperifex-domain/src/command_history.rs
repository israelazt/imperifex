//! The file of the commands run in the editor (T-D24): one command per line,
//! with its own line breaks written as `\n`.

/// How many commands are kept.
pub const KEPT: usize = 10_000;

/// Whether `command` is to be remembered after `last`. One that starts with a
/// space is not, as shells do for what must not be kept.
pub fn is_kept(command: &str, last: Option<&str>) -> bool {
    let secret = command.starts_with(char::is_whitespace);
    !secret && !command.trim().is_empty() && last != Some(command)
}

/// The line of the file for a command, with its end.
pub fn line(command: &str) -> String {
    let mut line = command.replace('\\', "\\\\").replace('\n', "\\n");
    line.push('\n');
    line
}

fn command(line: &str) -> String {
    let mut command = String::with_capacity(line.len());
    let mut characters = line.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            command.push(character);
            continue;
        }
        match characters.next() {
            Some('n') => command.push('\n'),
            Some(other) => command.push(other),
            None => command.push('\\'),
        }
    }
    command
}

/// What a file holds: the last [`KEPT`] commands, the oldest first. `true` if
/// there were more, so that the file can be cut.
pub fn read(text: &str) -> (Vec<String>, bool) {
    let lines: Vec<&str> = text.lines().filter(|line| !line.is_empty()).collect();
    let from = lines.len().saturating_sub(KEPT);
    let commands = lines[from..].iter().map(|line| command(line)).collect();
    (commands, from > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_command_comes_back_from_its_line_as_it_was() {
        for written in ["ls", "a\nb", r"cd C:\new\n", "x\\", "echo ñ €"] {
            let one = line(written);
            assert_eq!(one.matches('\n').count(), 1, "{written:?}");
            assert_eq!(read(&one).0, [written], "{written:?}");
        }
    }

    #[test]
    fn what_starts_with_a_space_is_empty_or_repeats_is_not_kept() {
        assert!(is_kept("ls", None));
        assert!(is_kept("ls", Some("pwd")));
        assert!(!is_kept("ls", Some("ls")));
        assert!(!is_kept(" secret", None));
        assert!(!is_kept("", None));
        assert!(!is_kept("\n\n", None));
    }

    #[test]
    fn only_the_last_ten_thousand_are_read_and_the_excess_is_said() {
        let text: String = (0..KEPT + 3).map(|n| line(&n.to_string())).collect();
        let (commands, excess) = read(&text);
        assert!(excess);
        assert_eq!(commands.len(), KEPT);
        assert_eq!(commands.first().map(String::as_str), Some("3"));
        assert!(!read("a\r\n\nb\n").1);
        assert_eq!(read("a\r\n\nb\n").0, ["a", "b"]);
    }
}
