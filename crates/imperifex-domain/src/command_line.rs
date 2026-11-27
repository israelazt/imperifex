//! A program and its arguments as the one line Windows starts a program with.
//! These are the rules most Windows programs read it back with; `cmd` has its
//! own, and its shell profile says so.

/// Whether an argument cannot go as it is.
fn needs_quotes(argument: &str) -> bool {
    argument.is_empty() || argument.contains([' ', '\t', '\n', '\x0b', '"'])
}

/// Adds one argument, quoted if it needs it. Backslashes only mean something
/// before a quote: there each one is doubled.
pub fn push_argument(line: &mut String, argument: &str) {
    if !line.is_empty() {
        line.push(' ');
    }
    if !needs_quotes(argument) {
        line.push_str(argument);
        return;
    }
    line.push('"');
    let mut backslashes = 0;
    for character in argument.chars() {
        if character == '\\' {
            backslashes += 1;
        } else {
            if character == '"' {
                line.extend(std::iter::repeat_n('\\', backslashes + 1));
            }
            backslashes = 0;
        }
        line.push(character);
    }
    // The closing quote must not be taken as part of the text.
    line.extend(std::iter::repeat_n('\\', backslashes));
    line.push('"');
}

pub fn command_line<S: AsRef<str>>(arguments: &[S]) -> String {
    let mut line = String::new();
    for argument in arguments {
        push_argument(&mut line, argument.as_ref());
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_arguments_go_as_they_are() {
        let line = command_line(&["cmd.exe", "/d", r"C:\carpeta\a.txt"]);
        assert_eq!(line, r"cmd.exe /d C:\carpeta\a.txt");
    }

    #[test]
    fn spaces_and_empty_arguments_are_quoted() {
        let line = command_line(&["a b", "", "c\td"]);
        assert_eq!(line, "\"a b\" \"\" \"c\td\"");
    }

    #[test]
    fn quotes_and_the_backslashes_before_them_are_escaped() {
        assert_eq!(command_line(&[r#"di "hola""#]), r#""di \"hola\"""#);
        assert_eq!(command_line(&[r#"a\"b"#]), r#""a\\\"b""#);
        assert_eq!(command_line(&[r"C:\con espacio\"]), r#""C:\con espacio\\""#);
    }
}
