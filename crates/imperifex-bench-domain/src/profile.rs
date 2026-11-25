//! How a terminal is opened and which processes are its own. It is data (in
//! `bench/terminals.json`), so measuring another terminal is one more entry.

use serde::{Deserialize, Serialize};

/// One terminal the bench knows how to measure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalProfile {
    /// How the command line calls it.
    pub name: String,
    /// The program that opens it; it may hold `%VARIABLES%` (see [`expand_env`]).
    pub exe: String,
    /// The arguments that make it open a new window.
    pub new_window_args: Vec<String>,
    /// Images that are the terminal itself.
    pub own_images: Vec<String>,
    /// Images that are the terminal's only when they descend from one of its own.
    pub helper_images: Vec<String>,
    /// The keys that open a tab, such as `Ctrl+Shift+T`; `None` if it has none.
    #[serde(default)]
    pub new_tab_keys: Option<String>,
}

/// Reads the list of profiles.
pub fn parse_profiles(json: &str) -> Result<Vec<TerminalProfile>, String> {
    serde_json::from_str(json)
        .map_err(|error| format!("los perfiles de terminal no se pueden leer: {error}"))
}

/// The profile with that name. The error says which names there are.
pub fn find<'a>(
    profiles: &'a [TerminalProfile],
    name: &str,
) -> Result<&'a TerminalProfile, String> {
    profiles.iter().find(|p| p.name == name).ok_or_else(|| {
        let known: Vec<&str> = profiles.iter().map(|p| p.name.as_str()).collect();
        format!(
            "no hay un perfil «{name}»; los que hay: {}",
            known.join(", ")
        )
    })
}

/// Replaces every `%NAME%` with what `lookup` answers; `%%` is one `%`. A
/// variable that does not exist, or a `%` left open, is an error.
pub fn expand_env(text: &str, lookup: &dyn Fn(&str) -> Option<String>) -> Result<String, String> {
    let mut expanded = String::new();
    let mut rest = text;
    while let Some((before, after)) = rest.split_once('%') {
        expanded.push_str(before);
        let (name, tail) = after
            .split_once('%')
            .ok_or_else(|| format!("falta cerrar un % en «{text}»"))?;
        if name.is_empty() {
            expanded.push('%');
        } else {
            let value = lookup(name).ok_or_else(|| format!("la variable %{name}% no existe"))?;
            expanded.push_str(&value);
        }
        rest = tail;
    }
    expanded.push_str(rest);
    Ok(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPOSITORY_PROFILES: &str = include_str!("../../../bench/terminals.json");

    fn variables(name: &str) -> Option<String> {
        (name == "APPS").then(|| r"D:\apps".to_string())
    }

    #[test]
    fn the_profiles_of_the_repository_can_be_read() {
        let profiles = parse_profiles(REPOSITORY_PROFILES).unwrap();
        let warp = find(&profiles, "warp").unwrap();
        assert_eq!(warp.own_images, ["warp.exe"]);
        assert_eq!(warp.helper_images, ["OpenConsole.exe", "conhost.exe"]);
        assert!(warp.new_window_args.is_empty());
        assert_eq!(warp.new_tab_keys.as_deref(), Some("Ctrl+Shift+T"));
        assert_eq!(warp.exe, r"%LOCALAPPDATA%\Programs\Warp\warp.exe");
    }

    #[test]
    fn broken_json_is_an_error_that_says_so() {
        assert!(parse_profiles("[{").unwrap_err().contains("perfiles"));
    }

    #[test]
    fn an_unknown_name_says_which_ones_there_are() {
        let profiles = parse_profiles(REPOSITORY_PROFILES).unwrap();
        let error = find(&profiles, "kitty").unwrap_err();
        assert!(error.contains("kitty") && error.contains("warp"), "{error}");
    }

    #[test]
    fn variables_are_replaced_and_plain_text_is_left_alone() {
        assert_eq!(
            expand_env(r"%APPS%\t.exe %APPS%", &variables).unwrap(),
            r"D:\apps\t.exe D:\apps"
        );
        assert_eq!(expand_env("t.exe", &variables).unwrap(), "t.exe");
    }

    #[test]
    fn a_variable_that_does_not_exist_is_an_error_with_its_name() {
        let error = expand_env(r"%NOPE%\t.exe", &variables).unwrap_err();
        assert!(error.contains("NOPE"), "{error}");
    }

    #[test]
    fn two_percent_signs_are_one_and_a_lone_one_is_an_error() {
        assert_eq!(expand_env("100%% de %%", &variables).unwrap(), "100% de %");
        assert!(expand_env("50% de", &variables).is_err());
    }
}
