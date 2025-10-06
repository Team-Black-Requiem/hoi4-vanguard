use std::{cell::RefCell, ops::Range, path::PathBuf};

use colored::Colorize;

/// A simple line/column position.
#[derive(Debug, Clone)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

/// A human-readable span (start and end positions).
#[derive(Debug, Clone)]
pub struct DisplayRange {
    pub start: Position,
    pub end: Position,
}

/// Convert byte offsets to line/column positions in the source string.
/// Handles edge cases where the range is out of bounds or at EOF.
pub fn byte_range_to_display_range(source: &str, byte_range: Range<usize>) -> DisplayRange {
    // Precompute byte offsets for each line start. This avoids indexing past bounds
    // and correctly handles files that don't end with a newline.
    let mut line_starts: Vec<usize> = Vec::new();
    line_starts.push(0);
    for (idx, ch) in source.char_indices() {
        if ch == '\n' {
            // Next character (if any) starts the following line
            if idx + 1 < source.len() {
                line_starts.push(idx + 1);
            }
        }
    }

    if line_starts.is_empty() {
        line_starts.push(0);
    }

    let total_lines = line_starts.len();

    let mut start_pos: Option<Position> = None;
    let mut end_pos: Option<Position> = None;

    for (line_idx, &line_start) in line_starts.iter().enumerate() {
        let line_number = line_idx + 1; // 1-based
        // Determine the end of this line in bytes. If this is not the last entry, use next start - 1.
        let line_end = if line_idx + 1 < line_starts.len() {
            line_starts[line_idx + 1]
        } else {
            source.len()
        };

        // If start hasn't been set and the byte_range.start falls within this line
        if start_pos.is_none() && byte_range.start >= line_start && byte_range.start <= line_end {
            // Compute column as number of chars from line_start to byte_range.start (1-based)
            let slice = &source[line_start..byte_range.start.min(source.len())];
            let column = slice.chars().count().saturating_add(1);
            start_pos = Some(Position { line: line_number, column });
        }

        if end_pos.is_none() && byte_range.end >= line_start && byte_range.end <= line_end {
            let slice = &source[line_start..byte_range.end.min(source.len())];
            let column = slice.chars().count().saturating_add(1);
            end_pos = Some(Position { line: line_number, column });
        }

        if start_pos.is_some() && end_pos.is_some() {
            break;
        }
    }

    // If the range is at or past EOF, clamp to the last line/column
    let last_line = total_lines.max(1);
    let last_line_start = *line_starts.last().unwrap_or(&0);
    let last_line_text = &source[last_line_start..];
    let last_line_len_chars = last_line_text.chars().count();

    DisplayRange {
        start: start_pos.unwrap_or(Position { line: last_line, column: last_line_len_chars + 1 }),
        end: end_pos.unwrap_or(Position { line: last_line, column: last_line_len_chars + 1 }),
    }
}

#[derive(Debug, Clone)]
pub struct Error(pub Range<usize>, pub String);

#[derive(Debug)]
pub struct ErrorContext {
    pub errors: RefCell<Vec<Error>>, // Already present
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            errors: RefCell::new(Vec::new())
        }
    }

    pub fn add_error(&self, error: Error) {
        self.errors.borrow_mut().push(error);
    }
}


pub fn print_error(source: &str, err: &Error) {
    let DisplayRange { start, end } = byte_range_to_display_range(source, err.0.clone());

    log::error!(
        "{}: {}\n  --> line {}:{}",
        "error".red().bold(),
        err.1,
        start.line,
        start.column
    );

    if let Some(line_text) = source.lines().nth(start.line.saturating_sub(1)) {
        log::error!("   |");
        log::error!("{:>3}| {}", start.line, line_text);

        let underline_start = start.column.saturating_sub(1).min(line_text.len());
        let underline_end = if start.line == end.line {
            end.column.saturating_sub(1).min(line_text.len())
        } else {
            underline_start
        };
        let underline_len = (underline_end as isize - underline_start as isize).max(1) as usize;

        let underline = format!(
            "{}{}",
            " ".repeat(underline_start),
            "^".repeat(underline_len)
        );
        log::error!("   | {}", underline.red());
    } else {
        log::error!("   | <line not available>");
    }

    // trailing label removed (was redundant)
}