//! One cell of the grid: what is written there and how many columns it takes.

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
}

impl Default for Cell {
    /// An empty cell: a space.
    fn default() -> Self {
        Self::new(' ', Span::Single)
    }
}

impl Cell {
    pub(super) fn new(base: char, span: Span) -> Self {
        Self {
            base,
            marks: None,
            span,
        }
    }

    /// The second column of a double character.
    pub(super) fn continuation() -> Self {
        Self::new(' ', Span::Continuation)
    }

    pub fn span(&self) -> Span {
        self.span
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
