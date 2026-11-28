//! What the clipboard holds on its way to the editor or to a program. Pasting
//! never runs anything by itself.

use crate::terminal::Modes;

/// What a tab becomes in the editor.
const TAB: &str = "    ";

/// `text` with one kind of line break, without those at its end, and without
/// control characters other than tabs: they would be keys, not text.
fn clean(text: &str) -> String {
    let one = text.replace("\r\n", "\n").replace('\r', "\n");
    let kept: String = one
        .chars()
        .filter(|&c| c == '\n' || c == '\t' || !c.is_control())
        .collect();
    kept.trim_end_matches('\n').to_string()
}

/// What is written in the editor.
pub fn for_editor(text: &str) -> String {
    clean(text).replace('\t', TAB)
}

/// What is done with a paste into a program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Paste {
    Nothing,
    Send(Vec<u8>),
    /// It has this many lines and the program does not take pastes: each break
    /// would be an `Enter`. It is sent only if the user asks again (T3.2).
    Ask(usize),
}

/// With the `modes` the program asked for; `asked` if the user already was.
pub fn for_program(text: &str, modes: Modes, asked: bool) -> Paste {
    let text = clean(text);
    if text.is_empty() {
        return Paste::Nothing;
    }
    let lines = text.split('\n').count();
    // A line break is what `Enter` sends.
    let typed = text.replace('\n', "\r");
    if modes.bracketed_paste {
        return Paste::Send(format!("\x1b[200~{typed}\x1b[201~").into_bytes());
    }
    if lines > 1 && !asked {
        return Paste::Ask(lines);
    }
    Paste::Send(typed.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sent(text: &str, bracketed_paste: bool, asked: bool) -> Paste {
        let modes = Modes {
            bracketed_paste,
            ..Modes::default()
        };
        for_program(text, modes, asked)
    }

    fn bytes(text: &str) -> Paste {
        Paste::Send(text.as_bytes().to_vec())
    }

    #[test]
    fn in_the_editor_breaks_are_one_kind_tabs_are_spaces_and_the_last_break_goes() {
        let found = for_editor("a\r\n\tb\rc\x1b[31m\x03\n\r\n");
        assert_eq!(found, "a\n    b\nc[31m");
    }

    #[test]
    fn one_line_goes_as_typed_and_never_with_the_enter_at_its_end() {
        assert_eq!(sent("rm -rf x\r\n", false, false), bytes("rm -rf x"));
    }

    #[test]
    fn a_program_that_takes_pastes_gets_it_marked_and_cannot_be_fooled_out_of_it() {
        // A text that carries the mark of the end would break out of the paste:
        // without its escape it is only text.
        let found = sent("a\x1b[201~\nb", true, false);
        assert_eq!(found, bytes("\x1b[200~a[201~\rb\x1b[201~"));
    }

    #[test]
    fn several_lines_into_a_program_without_pastes_are_asked_about_first() {
        assert_eq!(sent("a\nb\nc", false, false), Paste::Ask(3));
        assert_eq!(sent("a\nb", false, true), bytes("a\rb"));
    }

    #[test]
    fn nothing_worth_pasting_is_nothing() {
        for text in ["", "\r\n\n", "\x03"] {
            assert_eq!(sent(text, true, true), Paste::Nothing, "{text:?}");
        }
    }
}
