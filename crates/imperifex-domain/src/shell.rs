//! The shells Imperifex knows how to open, as data.

use crate::command_line::{command_line, push_argument};

/// How a shell wants the command it is given.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Quoting {
    /// As one argument, with the rules of most Windows programs.
    Argument,
    /// `cmd` takes the rest of its line: between two quotes, which `/s` makes
    /// it take off, and with nothing escaped.
    Cmd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellProfile {
    /// How the user names it.
    pub id: &'static str,
    pub name: &'static str,
    program: &'static str,
    /// What goes before a command so that it runs it and leaves.
    run_one: &'static [&'static str],
    quoting: Quoting,
}

const POWERSHELL_RUN: &[&str] = &["-NoLogo", "-NoProfile", "-Command"];

pub const PROFILES: [ShellProfile; 4] = [
    ShellProfile {
        id: "pwsh",
        name: "PowerShell 7",
        program: "pwsh.exe",
        run_one: POWERSHELL_RUN,
        quoting: Quoting::Argument,
    },
    ShellProfile {
        id: "powershell",
        name: "Windows PowerShell",
        program: "powershell.exe",
        run_one: POWERSHELL_RUN,
        quoting: Quoting::Argument,
    },
    ShellProfile {
        id: "cmd",
        name: "Símbolo del sistema",
        program: "cmd.exe",
        // `/d` skips what the registry would run first.
        run_one: &["/d", "/s", "/c"],
        quoting: Quoting::Cmd,
    },
    ShellProfile {
        id: "wsl",
        name: "WSL (distribución predeterminada)",
        program: "wsl.exe",
        // `--exec` hands the command to `sh` without another shell between.
        run_one: &["--exec", "sh", "-c"],
        quoting: Quoting::Argument,
    },
];

/// The names the user can give, for messages.
pub fn ids() -> String {
    PROFILES.map(|profile| profile.id).join(", ")
}

pub fn find(id: &str) -> Option<ShellProfile> {
    PROFILES.into_iter().find(|profile| profile.id == id)
}

impl ShellProfile {
    /// The line that makes this shell run `command` and leave.
    pub fn command_line(&self, command: &str) -> String {
        let mut line = String::from(self.program);
        let before = command_line(self.run_one);
        line.push(' ');
        line.push_str(&before);
        match self.quoting {
            Quoting::Argument => push_argument(&mut line, command),
            Quoting::Cmd => {
                line.push_str(" \"");
                line.push_str(command);
                line.push('"');
            }
        }
        line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(id: &str, command: &str) -> String {
        find(id).unwrap().command_line(command)
    }

    #[test]
    fn each_shell_gets_the_command_the_way_it_reads_it() {
        let command = r#"echo "a b""#;
        assert_eq!(
            line("powershell", command),
            r#"powershell.exe -NoLogo -NoProfile -Command "echo \"a b\"""#
        );
        assert_eq!(line("cmd", command), r#"cmd.exe /d /s /c "echo "a b"""#);
        assert_eq!(
            line("wsl", command),
            r#"wsl.exe --exec sh -c "echo \"a b\"""#
        );
        assert_eq!(
            line("pwsh", "exit"),
            "pwsh.exe -NoLogo -NoProfile -Command exit"
        );
    }

    #[test]
    fn names_are_unique_and_an_unknown_one_is_not_found() {
        for profile in PROFILES {
            assert_eq!(find(profile.id), Some(profile));
        }
        assert_eq!(find("fish"), None);
        assert_eq!(ids(), "pwsh, powershell, cmd, wsl");
    }
}
