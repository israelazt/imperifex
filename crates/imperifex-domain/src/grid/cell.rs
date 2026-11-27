//! One cell of the grid: what is written there, how it looks and how many
//! columns it takes.

use super::Style;

/// How many marks one cell keeps. What a program writes is not trusted: without
/// a limit, a stream of marks would grow one cell without end.
const MAX_MARKS: usize = 8;

/// How a cell takes part in the columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Span {
    #[default]
    Single,
    /// The first column of a character two columns wide.
    Double,
    /// The second column of the character on its left.
    Continuation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    base: char,
    /// Characters that take no column of their own (accents, joiners). Most
    /// cells have none, and then nothing is allocated.
    marks: Option<Box<str>>,
    span: Span,
    style: Style,
}

impl Default for Cell {
    /// An empty cell: a space with no style.
    fn default() -> Self {
        Self::new(' ', Span::Single, Style::default())
    }
}

impl Cell {
    pub(super) fn new(base: char, span: Span, style: Style) -> Self {
        Self {
            base,
            marks: None,
            span,
            style,
        }
    }

    /// An empty cell written with `style`.
    pub(super) fn blank(style: Style) -> Self {
        Self::new(' ', Span::Single, style)
    }

    /// The second column of a double character.
    pub(super) fn continuation(style: Style) -> Self {
        Self::new(' ', Span::Continuation, style)
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub(super) fn add_mark(&mut self, mark: char) {
        let old = self.marks.as_deref().unwrap_or("");
        if old.chars().count() < MAX_MARKS {
            self.marks = Some(format!("{old}{mark}").into_boxed_str());
        }
    }

    /// Adds what the cell shows to `text`; nothing for a continuation.
    pub fn push_to(&self, text: &mut String) {
        if self.span != Span::Continuation {
            text.push(self.base);
            text.push_str(self.marks.as_deref().unwrap_or(""));
        }
    }
}
