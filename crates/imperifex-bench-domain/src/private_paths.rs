//! Taking personal folders out of a text that will be published: the reports go
//! to a public repository and the errors in their notes carry paths (S5, D22).

fn same(a: char, b: char) -> bool {
    let plain = |c: char| if c == '/' { '\\' } else { c };
    plain(a).to_lowercase().eq(plain(b).to_lowercase())
}

/// How many bytes of the start of `text` are `folder`, if it starts with it as
/// whole path parts: `C:\Users\ana` is not in `C:\Users\anabel`.
fn length_at_start(text: &str, folder: &str) -> Option<usize> {
    let mut rest = text.chars();
    let mut length = 0;
    for wanted in folder.chars() {
        let found = rest.next().filter(|found| same(*found, wanted))?;
        length += found.len_utf8();
    }
    let goes_on = rest.next().is_some_and(char::is_alphanumeric);
    (!goes_on).then_some(length)
}

/// `text` with each folder of `folders`, given as (path, what to say instead),
/// replaced. Case and the direction of the slashes do not matter, and the
/// longest folder wins where several fit. An empty path changes nothing.
pub fn hide_folders(text: &str, folders: &[(String, String)]) -> String {
    let mut known: Vec<(&str, &str)> = folders
        .iter()
        .map(|(path, name)| (path.trim_end_matches(['\\', '/']), name.as_str()))
        .filter(|(path, _)| !path.is_empty())
        .collect();
    known.sort_by_key(|(path, _)| std::cmp::Reverse(path.chars().count()));

    let mut hidden = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(next) = rest.chars().next() {
        let here = known
            .iter()
            .find_map(|(path, name)| Some((length_at_start(rest, path)?, *name)));
        let (length, written) = match here {
            Some((length, name)) => (length, name),
            None => (next.len_utf8(), &rest[..next.len_utf8()]),
        };
        hidden.push_str(written);
        rest = &rest[length..];
    }
    hidden
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folders() -> Vec<(String, String)> {
        [
            (r"C:\Users\ana", "%USERPROFILE%"),
            (r"C:\Users\ana\AppData\Local\Temp\", "%TEMP%"),
            (r"C:\Users\ana\AppData\Local", "%LOCALAPPDATA%"),
        ]
        .map(|(path, name)| (path.to_string(), name.to_string()))
        .to_vec()
    }

    fn hidden(text: &str) -> String {
        hide_folders(text, &folders())
    }

    #[test]
    fn the_longest_folder_wins_over_those_it_is_inside() {
        assert_eq!(
            hidden(r"no hay C:\Users\ana\AppData\Local\Temp\bench\emit.json"),
            r"no hay %TEMP%\bench\emit.json"
        );
        assert_eq!(
            hidden(r"C:\Users\ana\AppData\Local\Programs\Warp\warp.exe"),
            r"%LOCALAPPDATA%\Programs\Warp\warp.exe"
        );
    }

    #[test]
    fn other_capitals_and_other_slashes_are_the_same_folder() {
        assert_eq!(
            hidden("c:/USERS/Ana/appdata/local/temp/x y C:/users/ANA"),
            "%TEMP%/x y %USERPROFILE%"
        );
    }

    #[test]
    fn a_folder_that_only_starts_alike_is_left_as_it_was() {
        assert_eq!(hidden(r"C:\Users\anabel\x"), r"C:\Users\anabel\x");
    }

    #[test]
    fn every_time_it_shows_up_it_is_hidden() {
        assert_eq!(
            hidden(r"de «C:\Users\ana» a C:\Users\ana."),
            "de «%USERPROFILE%» a %USERPROFILE%."
        );
    }

    #[test]
    fn an_empty_path_changes_nothing() {
        let empty = [(String::new(), "%TEMP%".to_string())];
        assert_eq!(hide_folders("tal cual: ñ", &empty), "tal cual: ñ");
    }
}
