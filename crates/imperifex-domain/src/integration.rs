//! Shell integration (T5.1): how a shell is opened so that it says when it
//! waits for a command, and how a command written outside it reaches it.

use crate::command_line::command_line;
use crate::terminal::Modes;

const POWERSHELL: &str = include_str!("integration/powershell.ps1");

/// `Shift`+`Enter`, going down and up, as the pseudoconsole's own way of
/// saying keys: virtual key, scan code, character, down, modifiers and count.
/// Windows `PowerShell` takes it as a new line of the command.
const SHIFT_ENTER: &str = "\x1b[13;28;13;1;16;1_\x1b[13;28;13;0;16;1_";

/// The line that opens the shell of `line` with the integration in it, without
/// touching the user's profile; `None` for a shell that has none yet.
pub fn integrated(line: &str) -> Option<String> {
    ["powershell.exe", "pwsh.exe"]
        .contains(&line)
        .then(|| command_line(&[line, "-NoExit", "-Command", POWERSHELL]))
}

/// What to send so that a shell with the integration runs `command`, with the
/// `modes` it asked for. A command of several lines goes as a paste if the
/// shell takes pastes, and else with `Shift`+`Enter` between them if it is a
/// `PowerShell`. `None` when it cannot be sent whole.
///
/// Control characters are left out: they would be keys, not text.
pub fn submission(command: &str, modes: Modes, powershell: bool) -> Option<Vec<u8>> {
    let lines: Vec<String> = command
        .split('\n')
        .map(|line| line.chars().filter(|c| !c.is_control()).collect())
        .collect();
    let text = match lines.as_slice() {
        [only] => only.clone(),
        _ if modes.bracketed_paste => format!("\x1b[200~{}\x1b[201~", lines.join("\r")),
        _ if powershell => lines.join(SHIFT_ENTER),
        _ => return None,
    };
    Some(format!("{text}\r").into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sent(command: &str, modes: Modes, powershell: bool) -> Option<String> {
        let bytes = submission(command, modes, powershell)?;
        String::from_utf8(bytes).ok()
    }

    #[test]
    fn only_the_powershells_have_the_integration_and_it_keeps_the_users_prompt() {
        for line in ["powershell.exe", "pwsh.exe"] {
            let found = integrated(line).expect("integrated");
            assert!(found.starts_with(&format!("{line} -NoExit -Command ")));
            assert!(found.contains("$function:prompt"));
            // The profile is neither skipped nor written to.
            assert!(!found.contains("-NoProfile") && !found.contains("$PROFILE"));
            assert!(found.len() < 30_000, "a command line has a limit");
        }
        assert_eq!(integrated("cmd.exe"), None);
        assert_eq!(integrated(r#""C:\My Tools\nu.exe""#), None);
    }

    #[test]
    fn one_line_goes_as_typed_with_enter_and_without_control_characters() {
        let found = sent("echo \"ñ\"\x1b[A\x03", Modes::default(), false);
        assert_eq!(found.as_deref(), Some("echo \"ñ\"[A\r"));
    }

    #[test]
    fn several_lines_are_pasted_where_the_shell_takes_pastes() {
        let modes = Modes {
            bracketed_paste: true,
            ..Modes::default()
        };
        for powershell in [true, false] {
            let found = sent("a\nb", modes, powershell);
            assert_eq!(found.as_deref(), Some("\x1b[200~a\rb\x1b[201~\r"));
        }
    }

    #[test]
    fn without_pastes_powershell_gets_shift_enter_and_others_nothing() {
        let found = sent("a\nb\nc", Modes::default(), true).expect("sent");
        assert_eq!(found, format!("a{SHIFT_ENTER}b{SHIFT_ENTER}c\r"));
        assert_eq!(sent("a\nb", Modes::default(), false), None);
    }
}
