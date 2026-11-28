//! Completing with `Tab` the path being written (T-D29): which word it is,
//! which folder to look in and what each name found would write.

/// The word before the cursor, read as a path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    /// Where it starts in the text, in bytes, with its quote if it has one.
    pub start: usize,
    /// The folder part as it was written, with its last separator.
    pub folder: String,
    /// What the names looked for start with.
    pub prefix: String,
    /// It is the first word of its line: a command, which `PowerShell` only
    /// runs from the current folder if it says so.
    pub command: bool,
}

/// The word that ends at `cursor`; `None` if there is none there.
pub fn word_before(text: &str, cursor: usize) -> Option<Word> {
    let before = text.get(..cursor)?;
    let (mut start, mut quote) = (0, None);
    for (at, character) in before.char_indices() {
        match quote {
            Some(open) if character == open => quote = None,
            Some(_) => {}
            None if character == '"' || character == '\'' => quote = Some(character),
            None if character.is_whitespace() => start = at + character.len_utf8(),
            None => {}
        }
    }
    let raw: String = before[start..]
        .chars()
        .filter(|&c| c != '"' && c != '\'')
        .collect();
    if raw.is_empty() {
        return None;
    }
    let cut = raw.rfind(['\\', '/']).map_or(0, |at| at + 1);
    let line = before[..start].rfind('\n').map_or(0, |at| at + 1);
    Some(Word {
        start,
        folder: raw[..cut].to_string(),
        prefix: raw[cut..].to_string(),
        command: before[line..start].trim().is_empty(),
    })
}

/// The folder to look in for `word`, with `current` that of the shell; `None`
/// when it cannot be known.
pub fn folder_of(word: &Word, current: Option<&str>) -> Option<String> {
    let written = word.folder.as_str();
    let has_drive = written.chars().nth(1) == Some(':');
    if has_drive || written.starts_with(r"\\") {
        return Some(written.to_string());
    }
    let current = current?;
    if written.starts_with(['\\', '/']) {
        // From the root of the current drive.
        let drive = current.get(..2)?;
        return Some(format!("{drive}{written}"));
    }
    let base = current.trim_end_matches('\\');
    Some(format!("{base}\\{written}"))
}

/// A path as `PowerShell` reads it whole: quoted when it has to be.
fn quoted(path: &str) -> String {
    let special = |c: char| c.is_whitespace() || "'\"()&;$`{},@#".contains(c);
    if path.contains(special) {
        format!("'{}'", path.replace('\'', "''"))
    } else {
        path.to_string()
    }
}

/// What each of `entries` (name, and whether it is a folder) would write for
/// `word`: those whose name starts with its prefix, upper or lower case alike,
/// by name.
pub fn options(word: &Word, entries: &[(String, bool)]) -> Vec<String> {
    let prefix = word.prefix.to_lowercase();
    let mut found: Vec<&(String, bool)> = entries
        .iter()
        .filter(|(name, _)| name.to_lowercase().starts_with(&prefix))
        .collect();
    found.sort_by_key(|(name, _)| name.to_lowercase());
    found
        .into_iter()
        .map(|(name, is_folder)| {
            let here = if word.command && word.folder.is_empty() {
                r".\"
            } else {
                ""
            };
            let end = if *is_folder { r"\" } else { "" };
            let path = quoted(&format!("{here}{}{name}{end}", word.folder));
            // A quoted command is only text to `PowerShell` without `&`.
            if word.command && !is_folder && path.starts_with('\'') {
                format!("& {path}")
            } else {
                path
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(text: &str) -> Option<Word> {
        word_before(text, text.len())
    }

    fn parts(text: &str) -> (usize, String, String, bool) {
        let found = word(text).expect("a word");
        (found.start, found.folder, found.prefix, found.command)
    }

    #[test]
    fn the_word_is_what_goes_from_the_last_space_outside_quotes_to_the_cursor() {
        let found = |start, folder: &str, prefix: &str, command| {
            (start, folder.to_string(), prefix.to_string(), command)
        };
        assert_eq!(parts("cd src\\gr"), found(3, "src\\", "gr", false));
        assert_eq!(parts("ca"), found(0, "", "ca", true));
        assert_eq!(parts("x\n  sub/"), found(4, "sub/", "", true));
        let quotes = "cd 'My Folder\\a b";
        assert_eq!(parts(quotes), found(3, "My Folder\\", "a b", false));
        assert_eq!(word("cd "), None);
        assert_eq!(word(""), None);
        assert_eq!(word_before("abc", 2).expect("a word").prefix, "ab");
    }

    #[test]
    fn the_folder_is_the_shells_unless_the_word_says_where() {
        let of = |text: &str, current| folder_of(&word(text).expect("a word"), current);
        let here = Some(r"C:\dev\app");
        assert_eq!(of("cd sr", here).as_deref(), Some(r"C:\dev\app\"));
        assert_eq!(of(r"cd ..\x", here).as_deref(), Some(r"C:\dev\app\..\"));
        assert_eq!(of(r"cd D:\a\b", here).as_deref(), Some(r"D:\a\"));
        assert_eq!(of(r"cd \Win", here).as_deref(), Some(r"C:\"));
        assert_eq!(
            of(r"cd \\pc\share\x", None).as_deref(),
            Some(r"\\pc\share\")
        );
        assert_eq!(of("cd sr", None), None);
        let root = of("cd W", Some(r"C:\"));
        assert_eq!(root.as_deref(), Some(r"C:\"));
    }

    fn entries(names: &[(&str, bool)]) -> Vec<(String, bool)> {
        let owned = |&(name, folder): &(&str, bool)| (name.to_string(), folder);
        names.iter().map(owned).collect()
    }

    #[test]
    fn the_names_that_start_so_come_in_order_and_folders_end_with_a_separator() {
        let found = entries(&[("src", true), ("Setup.md", false), ("docs", true)]);
        let word = word(r"cat .\s").expect("a word");
        assert_eq!(options(&word, &found), [r".\Setup.md", r".\src\"]);
    }

    #[test]
    fn a_name_with_spaces_or_signs_is_quoted_and_a_command_says_it_is_here() {
        let found = entries(&[("run it's.ps1", false)]);
        let argument = word("cat r").expect("a word");
        assert_eq!(options(&argument, &found), ["'run it''s.ps1'"]);
        let command = word("r").expect("a word");
        assert_eq!(options(&command, &found), [r"& '.\run it''s.ps1'"]);
        let plain = entries(&[("go.cmd", false)]);
        let command = word("g").expect("a word");
        assert_eq!(options(&command, &plain), [r".\go.cmd"]);
    }
}
