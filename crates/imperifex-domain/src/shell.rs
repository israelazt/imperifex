//! The shells Imperifex knows how to open, as data.

use crate::command_line::{command_line, push_argument};
use crate::i18n::{text, Language, Text};

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

/// What a tab opens when no shell was chosen, the first one that starts.
const BY_DEFAULT: [&str; 2] = ["pwsh", "powershell"];

/// The lines that open a shell to be used, with no command, in the order to be
/// tried: the one chosen, by its profile's name or as a program, or else those
/// by default.
pub fn opening_lines(chosen: Option<&str>) -> Vec<String> {
    let line = |name: &str| match find(name) {
        Some(profile) => profile.program.to_string(),
        None => command_line(&[name]),
    };
    match chosen {
        Some(name) => vec![line(name)],
        None => BY_DEFAULT.iter().map(|name| line(name)).collect(),
    }
}

/// What a tab calls the shell that `line` opened, one of [`opening_lines`]: a
/// short name for those of the profiles, and for another program that of its
/// file.
pub fn tab_name(line: &str, language: Language) -> String {
    let known = PROFILES.into_iter().find(|profile| profile.program == line);
    match known.map(|profile| profile.id) {
        Some("pwsh") => "PowerShell".into(),
        Some("powershell") => "Windows PowerShell".into(),
        Some("cmd") => text(language, Text::CommandPrompt).into(),
        Some("wsl") => "WSL".into(),
        _ => file_stem(line).into(),
    }
}

/// The name of the file of a program's path, quoted or not, with no extension.
fn file_stem(line: &str) -> &str {
    let path = line.trim_matches('"');
    let file = path.rsplit(['\\', '/']).next().unwrap_or(path);
    match file.rsplit_once('.') {
        Some((stem, _)) if !stem.is_empty() => stem,
        _ => file,
    }
}

/// The profile whose shell `line`, one of [`opening_lines`], opens; for
/// another program, the name of its file.
pub fn profile_of(line: &str) -> &str {
    let known = PROFILES.into_iter().find(|profile| profile.program == line);
    known.map_or_else(|| file_stem(line), |profile| profile.id)
}

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
    fn a_tab_opens_the_chosen_shell_or_else_the_best_one() {
        assert_eq!(opening_lines(None), ["pwsh.exe", "powershell.exe"]);
        assert_eq!(opening_lines(Some("cmd")), ["cmd.exe"]);
        assert_eq!(opening_lines(Some("nu.exe")), ["nu.exe"]);
        let spaced = opening_lines(Some(r"C:\My Tools\nu.exe"));
        assert_eq!(spaced, [r#""C:\My Tools\nu.exe""#]);
    }

    #[test]
    fn a_tab_is_named_by_its_profile_or_by_the_file_of_its_program() {
        let name = |line| tab_name(line, Language::English);
        assert_eq!(name("pwsh.exe"), "PowerShell");
        assert_eq!(name("powershell.exe"), "Windows PowerShell");
        assert_eq!(name("cmd.exe"), "Command Prompt");
        assert_eq!(
            tab_name("cmd.exe", Language::Spanish),
            "Símbolo del sistema"
        );
        assert_eq!(name("wsl.exe"), "WSL");
        assert_eq!(name("nu.exe"), "nu");
        assert_eq!(name(r#""C:\My Tools\nu.v2.exe""#), "nu.v2");
        assert_eq!(name(".profile"), ".profile");
    }

    #[test]
    fn the_profile_of_a_line_is_its_id_or_the_name_of_its_file() {
        assert_eq!(profile_of("powershell.exe"), "powershell");
        assert_eq!(profile_of("wsl.exe"), "wsl");
        assert_eq!(profile_of("nu.exe"), "nu");
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
