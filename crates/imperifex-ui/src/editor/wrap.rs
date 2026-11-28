//! Cutting what the editor shows into the rows of its box.

use imperifex_domain::view::EditorView;

/// How many rows of a command are seen at once: the box does not take the whole
/// window.
const MOST_ROWS: usize = 10;

/// A line in pieces of `per_row` characters; an empty one is one empty piece.
fn pieces(line: &str, per_row: usize) -> Vec<String> {
    let characters: Vec<char> = line.chars().collect();
    if characters.is_empty() {
        return vec![String::new()];
    }
    characters
        .chunks(per_row.max(1))
        .map(|piece| piece.iter().collect())
        .collect()
}

pub fn all(lines: &[String], per_row: usize) -> Vec<String> {
    lines
        .iter()
        .flat_map(|line| pieces(line, per_row))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// The rows seen.
    pub rows: Vec<String>,
    /// The row of the cursor among them, and how many characters are before it.
    pub cursor: (usize, usize),
    /// What of the suggestion fits after the cursor in its row.
    pub suggestion: String,
}

pub fn command(view: &EditorView, per_row: usize) -> Command {
    let per_row = per_row.max(1);
    let (line, column) = view.cursor;
    let mut rows = Vec::new();
    let mut cursor = (0, 0);
    for (at, text) in view.lines.iter().enumerate() {
        let first = rows.len();
        rows.extend(pieces(text, per_row));
        if at == line {
            // At the end of a full row the cursor stays on it.
            let full_end = column > 0 && column % per_row == 0 && column == text.chars().count();
            let row = column / per_row - usize::from(full_end);
            cursor = (first + row, column - row * per_row);
        }
    }
    // The rows seen end late enough to have the cursor's.
    let from = (cursor.0 + 1).saturating_sub(MOST_ROWS);
    let to = rows.len().min(from + MOST_ROWS);
    let seen = rows.drain(from..to).collect();
    let first_line = view.suggestion.split('\n').next().unwrap_or("");
    Command {
        rows: seen,
        cursor: (cursor.0 - from, cursor.1),
        suggestion: first_line
            .chars()
            .take(per_row - cursor.1.min(per_row))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(lines: &[&str], cursor: (usize, usize), suggestion: &str) -> EditorView {
        EditorView {
            lines: lines.iter().map(ToString::to_string).collect(),
            cursor,
            suggestion: suggestion.into(),
            ..EditorView::default()
        }
    }

    #[test]
    fn long_lines_are_cut_into_rows_and_the_cursor_follows() {
        let found = command(&view(&["abcdefg", "", "xy"], (0, 5), ""), 3);
        assert_eq!(found.rows, ["abc", "def", "g", "", "xy"]);
        assert_eq!(found.cursor, (1, 2));
        assert_eq!(command(&view(&["", "xy"], (1, 2), ""), 3).cursor, (1, 2));
    }

    #[test]
    fn at_the_end_of_a_full_row_the_cursor_stays_on_it() {
        assert_eq!(command(&view(&["abcdef"], (0, 6), ""), 3).cursor, (1, 3));
        // In the middle of the line it goes to the next row.
        assert_eq!(command(&view(&["abcdef"], (0, 3), ""), 3).cursor, (1, 0));
    }

    #[test]
    fn a_long_command_shows_the_ten_rows_that_have_the_cursor() {
        let lines: Vec<String> = (0..25).map(|n| n.to_string()).collect();
        let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
        let last = command(&view(&lines, (24, 2), ""), 40);
        assert_eq!((last.rows.len(), last.cursor.0), (10, 9));
        assert_eq!(last.rows[0], "15");
        let first = command(&view(&lines, (0, 0), ""), 40);
        assert_eq!((first.rows[0].as_str(), first.cursor.0), ("0", 0));
        assert_eq!(first.rows.len(), 10);
    }

    #[test]
    fn only_what_fits_in_its_row_of_the_first_line_is_suggested() {
        let found = command(&view(&["ab"], (0, 2), "cdefgh\nmore"), 5);
        assert_eq!(found.suggestion, "cde");
        let full = command(&view(&["abcde"], (0, 5), "f"), 5);
        assert_eq!(full.suggestion, "");
    }
}
