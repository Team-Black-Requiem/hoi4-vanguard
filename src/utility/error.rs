use std::ops::Range;

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
pub fn byte_range_to_display_range(source: &str, byte_range: Range<usize>) -> DisplayRange {
    let mut current_offset = 0;
    let mut line_number = 1;
    let mut start_pos = None;
    let mut end_pos = None;

    for line in source.lines() {
        let line_len = line.len() + 1; // include '\n'
        let line_start = current_offset;
        let line_end = current_offset + line_len;

        if start_pos.is_none() && byte_range.start < line_end {
            start_pos = Some(Position {
                line: line_number,
                column: byte_range.start - line_start + 1,
            });
        }

        if end_pos.is_none() && byte_range.end <= line_end {
            end_pos = Some(Position {
                line: line_number,
                column: byte_range.end - line_start + 1,
            });
        }

        if start_pos.is_some() && end_pos.is_some() {
            break;
        }

        current_offset += line_len;
        line_number += 1;
    }

    DisplayRange {
        start: start_pos.unwrap_or(Position { line: 0, column: 0 }),
        end: end_pos.unwrap_or(Position { line: 0, column: 0 }),
    }
}

#[derive(Debug, Clone)]
pub struct Error(pub Range<usize>, pub String);


pub fn print_error(source: &str, err: &Error) {
    let DisplayRange { start, end } = byte_range_to_display_range(source, err.0.clone());

    println!(
        "{}: {}\n  --> line {}:{}",
        "error".red().bold(),
        err.1,
        start.line,
        start.column
    );

    if let Some(line_text) = source.lines().nth(start.line - 1) {
        println!("   |\n{:>3} | {}", start.line, line_text);

        let underline_len = if start.line == end.line {
            (end.column - start.column).max(1)
        } else {
            1
        };

        let underline = " ".repeat(start.column - 1) + &"^".repeat(underline_len);
        println!("   | {}", underline.red());
    } else {
        println!("   | <line not available>");
    }

    println!();
}
