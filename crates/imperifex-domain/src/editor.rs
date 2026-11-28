//! The command editor of Imperifex (T5.6): a text of one or several lines with
//! a cursor, the history of the commands run and the suggestion taken from it.

use crate::completion::{word_before, Word};
use crate::input::{KeyInput, NamedKey};

/// What `Tab` writes with no word to complete.
const TAB: &str = "    ";

/// What a key asks of the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorKey {
    Insert(String),
    NewLine,
    Run,
    /// `Tab`: complete what is before the cursor; with `Shift`, the one before
    /// among those found.
    Complete {
        back: bool,
    },
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Clear,
    WordLeft,
    WordRight,
    DeleteWordLeft,
    DeleteWordRight,
    /// `Ctrl`+`C`.
    Interrupt,
}

/// What a key asks of the editor; `None` for one that asks nothing.
pub fn editor_key(input: &KeyInput) -> Option<EditorKey> {
    let by_words = |word, character| if input.ctrl { word } else { character };
    Some(match input.key {
        Some(NamedKey::Enter) if input.shift => EditorKey::NewLine,
        Some(NamedKey::Enter) => EditorKey::Run,
        Some(NamedKey::Tab) => EditorKey::Complete { back: input.shift },
        Some(NamedKey::Backspace) => by_words(EditorKey::DeleteWordLeft, EditorKey::Backspace),
        Some(NamedKey::Delete) => by_words(EditorKey::DeleteWordRight, EditorKey::Delete),
        Some(NamedKey::Escape) => EditorKey::Clear,
        Some(NamedKey::Left) => by_words(EditorKey::WordLeft, EditorKey::Left),
        Some(NamedKey::Right) => by_words(EditorKey::WordRight, EditorKey::Right),
        Some(NamedKey::Up) => EditorKey::Up,
        Some(NamedKey::Down) => EditorKey::Down,
        Some(NamedKey::Home) => EditorKey::Home,
        Some(NamedKey::End) => EditorKey::End,
        Some(NamedKey::Insert | NamedKey::PageUp | NamedKey::PageDown | NamedKey::Function(_)) => {
            return None;
        }
        None => {
            let text = input.text.as_deref().filter(|_| !input.alt)?;
            if text == "\u{3}" {
                return Some(EditorKey::Interrupt);
            }
            let writes = !text.is_empty() && !text.chars().any(char::is_control);
            return writes.then(|| EditorKey::Insert(text.into()));
        }
    })
}

/// What came of a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Unchanged,
    Changed,
    /// Run this command. The text stays until [`Editor::sent`].
    Run(String),
    /// `Ctrl`+`C` with nothing written: it is for the shell.
    Interrupt,
    /// This word is to be completed: [`Editor::offer`] what it can be.
    Complete(Word),
}

#[derive(Debug, Clone, Default)]
pub struct Editor {
    text: String,
    /// In bytes, always between two characters.
    cursor: usize,
    /// The commands run, the oldest first.
    history: Vec<String>,
    /// Which one is being looked at, and what was written before looking.
    looking: Option<(usize, String)>,
    /// The completions being gone through, while no other key is pressed.
    completing: Option<Completing>,
}

#[derive(Debug, Clone)]
struct Completing {
    /// Where the completed word starts.
    start: usize,
    options: Vec<String>,
    at: usize,
}

impl Editor {
    pub fn new(history: Vec<String>) -> Self {
        Self {
            history,
            ..Self::default()
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// The command run last.
    pub fn last(&self) -> Option<&str> {
        self.history.last().map(String::as_str)
    }

    /// The line the cursor is on, from 0, and how many characters of it are
    /// before the cursor.
    pub fn cursor(&self) -> (usize, usize) {
        let before = &self.text[..self.cursor];
        let line = before.matches('\n').count();
        let start = before.rfind('\n').map_or(0, |at| at + 1);
        (line, before[start..].chars().count())
    }

    /// What the most recent command that starts with what is written has after
    /// it; only with the cursor at the end.
    pub fn suggestion(&self) -> &str {
        if self.text.is_empty() || self.cursor < self.text.len() {
            return "";
        }
        self.history
            .iter()
            .rev()
            .find(|past| past.len() > self.text.len() && past.starts_with(&self.text))
            .map_or("", |past| &past[self.text.len()..])
    }

    /// The command given by [`Outcome::Run`] reached the shell: it joins the
    /// history if `kept`, and the editor is empty again.
    pub fn sent(&mut self, kept: bool) {
        let command = std::mem::take(&mut self.text);
        if kept {
            self.history.push(command);
        }
        self.cursor = 0;
        self.looking = None;
    }

    pub fn apply(&mut self, key: EditorKey) -> Outcome {
        let before = (self.text.clone(), self.cursor);
        if !matches!(key, EditorKey::Complete { .. }) {
            self.completing = None;
        }
        match key {
            EditorKey::Complete { back } => return self.tab(back),
            EditorKey::Run => return Outcome::Run(self.text.clone()),
            EditorKey::Interrupt if self.text.is_empty() => return Outcome::Interrupt,
            EditorKey::Interrupt | EditorKey::Clear => self.set(String::new()),
            EditorKey::Insert(text) => self.insert(&text),
            EditorKey::NewLine => self.insert("\n"),
            EditorKey::Backspace => {
                if let Some(from) = self.before(self.cursor) {
                    self.remove(from, self.cursor);
                }
            }
            EditorKey::Delete => {
                if let Some(to) = self.after(self.cursor) {
                    self.remove(self.cursor, to);
                }
            }
            EditorKey::Left => self.cursor = self.before(self.cursor).unwrap_or(0),
            EditorKey::Right => self.right(),
            EditorKey::WordLeft => self.cursor = self.word_before(),
            EditorKey::WordRight => self.cursor = self.word_after(),
            EditorKey::DeleteWordLeft => self.remove(self.word_before(), self.cursor),
            EditorKey::DeleteWordRight => {
                let (from, to) = (self.cursor, self.word_after());
                self.remove(from, to);
            }
            EditorKey::Home => self.cursor = self.line_start(),
            EditorKey::End => self.cursor = self.line_end(),
            EditorKey::Up => self.up(),
            EditorKey::Down => self.down(),
        }
        if before == (self.text.clone(), self.cursor) {
            Outcome::Unchanged
        } else {
            Outcome::Changed
        }
    }

    /// Writes `text` where the cursor is, as a paste does.
    pub fn paste(&mut self, text: &str) {
        self.completing = None;
        self.insert(text);
    }

    /// Goes on to the next completion, or asks to complete the word before the
    /// cursor; with no word there, it indents.
    fn tab(&mut self, back: bool) -> Outcome {
        if let Some(mut completing) = self.completing.take() {
            let count = completing.options.len();
            let step = if back { count - 1 } else { 1 };
            let next = (completing.at + step) % count;
            self.write_option(&completing, next);
            completing.at = next;
            self.completing = Some(completing);
            return Outcome::Changed;
        }
        match word_before(&self.text, self.cursor) {
            Some(word) => Outcome::Complete(word),
            None => {
                self.insert(TAB);
                Outcome::Changed
            }
        }
    }

    /// What the word that starts at `start` can be, the first of which is
    /// written; the next `Tab` writes the next. `false` if there is nothing.
    pub fn offer(&mut self, start: usize, options: Vec<String>) -> bool {
        if options.is_empty() || start > self.cursor {
            return false;
        }
        let completing = Completing {
            start,
            options,
            at: 0,
        };
        self.write_option(&completing, 0);
        self.completing = Some(completing);
        true
    }

    fn write_option(&mut self, completing: &Completing, at: usize) {
        if let Some(option) = completing.options.get(at) {
            let (start, end) = (completing.start, self.cursor);
            self.text.replace_range(start..end, option);
            self.cursor = start + option.len();
            self.looking = None;
        }
    }

    /// Puts the cursor on the line `line`, after `characters` of it, or as
    /// close as there is.
    pub fn move_to(&mut self, line: usize, characters: usize) {
        let mut start = 0;
        for _ in 0..line {
            match self.text[start..].find('\n') {
                Some(at) => start += at + 1,
                None => break,
            }
        }
        let end = self.text[start..]
            .find('\n')
            .map_or(self.text.len(), |at| start + at);
        self.cursor = self.along(start, end, characters);
    }

    /// Where the word before the cursor starts, past the spaces between.
    fn word_before(&self) -> usize {
        let before = self.text[..self.cursor].trim_end();
        before
            .rfind(char::is_whitespace)
            .and_then(|at| self.after(at))
            .unwrap_or(0)
    }

    /// Where the word after the cursor ends, past the spaces between.
    fn word_after(&self) -> usize {
        let rest = &self.text[self.cursor..];
        let word = rest.trim_start();
        let spaces = rest.len() - word.len();
        let end = word.find(char::is_whitespace).unwrap_or(word.len());
        self.cursor + spaces + end
    }

    /// Replaces the text, with the cursor at its end.
    fn set(&mut self, text: String) {
        self.cursor = text.len();
        self.text = text;
        self.looking = None;
    }

    fn insert(&mut self, text: &str) {
        self.text.insert_str(self.cursor, text);
        self.cursor += text.len();
        // What is edited is a command of its own, no longer one looked at.
        self.looking = None;
    }

    fn remove(&mut self, from: usize, to: usize) {
        self.text.replace_range(from..to, "");
        self.cursor = from;
        self.looking = None;
    }

    /// Where the character before `at` starts.
    fn before(&self, at: usize) -> Option<usize> {
        self.text[..at].char_indices().next_back().map(|(i, _)| i)
    }

    /// Where the character at `at` ends.
    fn after(&self, at: usize) -> Option<usize> {
        let character = self.text[at..].chars().next()?;
        Some(at + character.len_utf8())
    }

    /// At the end, it takes the suggestion.
    fn right(&mut self) {
        match self.after(self.cursor) {
            Some(next) => self.cursor = next,
            None => {
                let rest = self.suggestion().to_string();
                if !rest.is_empty() {
                    self.insert(&rest);
                }
            }
        }
    }

    fn line_start(&self) -> usize {
        self.text[..self.cursor].rfind('\n').map_or(0, |at| at + 1)
    }

    fn line_end(&self) -> usize {
        self.text[self.cursor..]
            .find('\n')
            .map_or(self.text.len(), |at| self.cursor + at)
    }

    /// The place `characters` into the line from `start` to `end`.
    fn along(&self, start: usize, end: usize, characters: usize) -> usize {
        self.text[start..end]
            .char_indices()
            .nth(characters)
            .map_or(end, |(at, _)| start + at)
    }

    /// A line up, or on the first one, the command before in the history.
    fn up(&mut self) {
        let start = self.line_start();
        if start == 0 {
            self.older();
            return;
        }
        let column = self.cursor().1;
        let end = start - 1;
        let above = self.text[..end].rfind('\n').map_or(0, |at| at + 1);
        self.cursor = self.along(above, end, column);
    }

    /// A line down, or on the last one, back toward what was being written.
    fn down(&mut self) {
        let end = self.line_end();
        if end == self.text.len() {
            self.newer();
            return;
        }
        let column = self.cursor().1;
        let below = end + 1;
        let below_end = self.text[below..]
            .find('\n')
            .map_or(self.text.len(), |at| below + at);
        self.cursor = self.along(below, below_end, column);
    }

    fn older(&mut self) {
        let (at, written) = match self.looking.take() {
            Some((at, written)) => (at.checked_sub(1), written),
            None => (self.history.len().checked_sub(1), self.text.clone()),
        };
        match at {
            Some(at) => self.look(at, written),
            // Past the oldest it stays; with no history nothing happens.
            None if !self.history.is_empty() => self.look(0, written),
            None => {}
        }
    }

    fn newer(&mut self) {
        let Some((at, written)) = self.looking.take() else {
            return;
        };
        if at + 1 < self.history.len() {
            self.look(at + 1, written);
        } else {
            self.set(written);
        }
    }

    fn look(&mut self, at: usize, written: String) {
        let command = self.history.get(at).cloned().unwrap_or_default();
        self.set(command);
        self.looking = Some((at, written));
    }
}

#[cfg(test)]
mod tests;
