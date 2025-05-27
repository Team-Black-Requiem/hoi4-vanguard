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
    let mut current_offset = 0;
    let mut line_number = 1;
    let mut start_pos = None;
    let mut end_pos = None;

    for (i, line) in source.lines().enumerate() {
        // Handle both \n and \r\n endings
        let line_len = line.len();
        let line_ending_len = if source[current_offset + line_len..].starts_with("\r\n") { 2 } else { 1 };
        let total_line_len = line_len + line_ending_len;
        let line_start = current_offset;
        let line_end = current_offset + total_line_len;

        if start_pos.is_none() && byte_range.start < line_end {
            start_pos = Some(Position {
                line: line_number,
                column: (byte_range.start - line_start + 1).max(1),
            });
        }

        if end_pos.is_none() && byte_range.end <= line_end {
            end_pos = Some(Position {
                line: line_number,
                column: (byte_range.end - line_start + 1).max(1),
            });
        }

        if start_pos.is_some() && end_pos.is_some() {
            break;
        }

        current_offset += total_line_len;
        line_number += 1;
    }

    // If the range is at or past EOF, clamp to the last line/column
    let last_line = source.lines().count().max(1);
    let last_line_len = source.lines().last().map(|l| l.len()).unwrap_or(0);

    DisplayRange {
        start: start_pos.unwrap_or(Position { line: last_line, column: last_line_len + 1 }),
        end: end_pos.unwrap_or(Position { line: last_line, column: last_line_len + 1 }),
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

    log::error!("   | {}", "error".red().bold());
}