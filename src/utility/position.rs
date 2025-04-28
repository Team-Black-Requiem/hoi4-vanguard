use std::{
    collections::HashMap,
    fmt,
    io::{self, Write},
    hash::{Hash, Hasher},
    path::{Path, PathBuf}
};

use serde::{Deserialize, Serialize};

// Bit manipulation functions
const fn pown32(n: i32) -> i32 {
    if n == 0 { 1 } else { pown32(n - 1) | (1 << (n - 1)) }
}

const fn pown64(n: i64) -> i64 {
    if n == 0 { 1 } else { pown64(n - 1) | (1 << (n - 1)) }
}

const fn mask32(m: i32, n: i32) -> i32 {
    pown32(n) << m
}

const fn mask64(m: i64, n: i64) -> i64 {
    pown64(n) << m
}

// position struct
#[derive(Clone, Copy, PartialOrd, Ord, Eq)]
pub struct Pos {
    code: u32,
}

impl Pos {
    // Constants used in the bit manipulation
    const COLUMN_BIT_COUNT: i32 = 11;
    const LINE_BIT_COUNT: i32 = 21;
    const POS_COLUMN_MASK: u32 = (1 << Self::COLUMN_BIT_COUNT) - 1; // 0b00000000000000000000111111111111
    const LINE_COLUMN_MASK: i32 = ((1 << Self::LINE_BIT_COUNT) - 1) << Self::COLUMN_BIT_COUNT; // 0b11111111111111111111000000000000

    // Constructor
    pub fn new(line: i32, column: i32) -> Self {
        assert!((0..(1 << Self::LINE_BIT_COUNT)).contains(&line), "Line value out of range");
        assert!((0..(1 << Self::COLUMN_BIT_COUNT)).contains(&column), "Column value out of range");
    
        let code = (column as u32 & Self::POS_COLUMN_MASK)
            | ((line as u32) << Self::COLUMN_BIT_COUNT);
        Pos { code }
    }

    // Shift right equivalent (bitwise shift)
    pub fn lsr(x: i32, y: i32) -> i32 {
        (x as u32 >> y) as i32
    }

    // Accessors
    #[inline]
    pub fn line(&self) -> i32 {
        (self.code >> Self::COLUMN_BIT_COUNT) as i32
    }
    
    #[inline]
    pub fn column(&self) -> i32 {
        (self.code & Self::POS_COLUMN_MASK) as i32
    }

    pub fn encoding(&self) -> u32 {
        self.code
    }

    // Static method for decoding an encoded position
    pub fn decode(code: u32) -> Self {
        Pos { code }
    }

    pub fn pos_eq(&self, other: &Pos) -> bool {
        self.line() == other.line() && self.column() == other.column()
    }

    pub fn pos_gt(&self, other: &Pos) -> bool {
        self.line() > other.line() || (self.line() == other.line() && self.column() > other.column())
    }

    pub fn pos_geq(&self, other: &Pos) -> bool {
        self.line() > other.line() || (self.line() == other.line() && self.column() >= other.column())
    }

    pub fn pos_lt(&self, other: &Pos) -> bool {
        self.line() < other.line() || (self.line() == other.line() && self.column() < other.column())
    }

    /// Helper method for a "less than or equal" check
    pub fn pos_leq(&self, other: &Pos) -> bool {
        self.line() < other.line() || (self.line() == other.line() && self.column() <= other.column())
    }

    pub fn validate(&self) {
        assert!(self.line() >= 0 && self.line() < (1 << Self::LINE_BIT_COUNT), "Invalid line value");
        assert!(self.column() >= 0 && self.column() < (1 << Self::COLUMN_BIT_COUNT), "Invalid column value");
    }

}

impl PartialEq for Pos {
    fn eq(&self, other: &Self) -> bool {
        self.line() == other.line() && self.column() == other.column()
    }
}

impl Hash for Pos {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.code.hash(h);
    }
}

// Implementing the Display trait
impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.line(), self.column())
    }
}

// Implementing custom Debug trait
impl std::fmt::Debug for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.line(), self.column())
    }
}

pub fn output_pos<W: Write>(writer: &mut W, pos: &Pos) -> io::Result<()> {
    writeln!(writer, "{}", pos)
}


#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Range {
    code: u64,        // Encodes start/end line, column, height, etc.
    // come up with solution to tie into file index and union system
}

impl Range {
    // Bitmask and shift constants for encoding/decoding positions
    const IS_SYNTHETIC_MASK: u64 = 1 << 63;

    pub fn new(start_line: i32, start_column: i32, end_line: i32, end_column: i32) -> Self {
        assert!((0..(1 << 21)).contains(&start_line), "Start line out of range");
        assert!((0..(1 << 11)).contains(&start_column), "Start column out of range");
        assert!(end_line >= start_line, "End line must be >= start line");
        assert!((0..(1 << 11)).contains(&end_column), "End column out of range");
    
        let code = ((start_line as u64) & 0x1FFFFF)
            | (((start_column as u64) & 0x7FF) << 21)
            | (((end_line as u64 - start_line as u64) & 0xFFFFF) << 32)
            | (((end_column as u64) & 0x7FF) << 52);
    
        Range { code }
    }

    // Check if this range is synthetic
    pub fn is_synthetic(&self) -> bool {
        (self.code & Self::IS_SYNTHETIC_MASK) != 0
    }

    // Mark this range as synthetic
    pub fn make_synthetic(&mut self) {
        self.code |= Self::IS_SYNTHETIC_MASK;
    }

    // Accessor for the encoded code
    pub fn code(&self) -> u64 {
        self.code
    }

    // Start and end positions as Pos (assumes Pos struct is implemented)
    pub fn start_pos(&self) -> Pos {
        Pos::new(self.start_line(), self.start_column())
    }

    pub fn end_pos(&self) -> Pos {
        Pos::new(self.end_line(), self.end_column())
    }

    // Accessors for start and end line/column values
    #[inline]
    pub fn start_line(&self) -> i32 {
        (self.code & 0x1FFFFF) as i32
    }
    
    #[inline]
    pub fn start_column(&self) -> i32 {
        ((self.code >> 21) & 0x7FF) as i32
    }
    
    #[inline]
    pub fn end_line(&self) -> i32 {
        (((self.code >> 32) & 0xFFFFF) as i32) + self.start_line()
    }
    
    #[inline]
    pub fn end_column(&self) -> i32 {
        ((self.code >> 52) & 0x7FF) as i32
    }

    /// Short string format for displaying range positions only
    pub fn to_short_string(&self) -> String {
        format!("({}:{})--({}:{})", 
                self.start_line(), 
                self.start_column(), 
                self.end_line(), 
                self.end_column())
    }
    
    /// Check if this range completely contains another range
    pub fn contains_range(&self, other: &Range) -> bool {
            self.start_pos().pos_leq(&other.start_pos())
            && self.end_pos().pos_geq(&other.end_pos())
    }

    /// Check if this range contains a specific position
    pub fn contains_pos(&self, pos: &Pos) -> bool {
        self.start_pos().pos_leq(pos) && self.end_pos().pos_geq(pos)
    }

    pub fn validate(&self) {
        assert!(self.start_line() >= 0 && self.start_line() < (1 << 21), "Invalid start line");
        assert!(self.start_column() >= 0 && self.start_column() < (1 << 11), "Invalid start column");
        assert!(self.end_line() >= self.start_line(), "End line must be >= start line");
        assert!(self.end_column() >= 0 && self.end_column() < (1 << 11), "Invalid end column");
    }

    /// Union of two ranges, if they are within the same file and layer
    pub fn union(&self, other: &Range) -> Option<Range> {
        let start_pos = if self.start_pos().pos_leq(&other.start_pos()) {
            self.start_pos()
        } else {
            other.start_pos()
        };

        let end_pos = if self.end_pos().pos_geq(&other.end_pos()) {
            self.end_pos()
        } else {
            other.end_pos()
        };

        Some(Range::new(
            start_pos.line(),
            start_pos.column(),
            end_pos.line(),
            end_pos.column(),
        ))
    }

    /// Check if this range occurs entirely before a given position
    pub fn range_before_pos(&self, pos: &Pos) -> bool {
        self.end_pos().pos_lt(pos)
    }

}

// Implementing Display trait for easy printing
impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}:{}--{}:{})",
            self.start_line(),
            self.start_column(),
            self.end_line(),
            self.end_column()
        )
    }
}
/*
pub fn mk_file_index_range(file_index: i16, start_pos: Pos, end_pos: Pos) -> Range {
    Range::new(file_index, start_pos.line(), start_pos.column(), end_pos.line(), end_pos.column())
}

pub fn union_ranges(range1: &Range, range2: &Range) -> Range {
    if range1.file_index() != range2.file_index() {
        return *range2;  // Different files, so no meaningful union can be made.
    }
    
    let start_pos = if range1.start_pos() < range2.start_pos() {
        range1.start_pos()
    } else {
        range2.start_pos()
    };
    
    let end_pos = if range1.end_pos() > range2.end_pos() {
        range1.end_pos()
    } else {
        range2.end_pos()
    };
    
    Range::new(
        range1.file_index(),
        start_pos.line(),
        start_pos.column(),
        end_pos.line(),
        end_pos.column()
    )
}

pub fn range_contains_range(outer: &Range, inner: &Range) -> bool {
    outer.file_index() == inner.file_index()
        && outer.start_pos() <= inner.start_pos()
        && outer.end_pos() >= inner.end_pos()
}

pub fn range_contains_pos(range: &Range, pos: &Pos) -> bool {
    range.start_pos() <= *pos && range.end_pos() >= *pos
}

pub fn range_before_pos(range: &Range, pos: &Pos) -> bool {
    range.end_pos() < *pos
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_creation() {
        let pos = Pos::new(10, 15);
        assert_eq!(pos.line(), 10);
        assert_eq!(pos.column(), 15);
    }

    #[test]
    fn test_range_creation() {
        let range = Range::new(10, 15, 20, 25);
        assert_eq!(range.start_line(), 10);
        assert_eq!(range.start_column(), 15);
        assert_eq!(range.end_line(), 20);
        assert_eq!(range.end_column(), 25);
    }

    #[test]
    fn test_range_union() {
        let range1 = Range::new(10, 10, 20, 20);
        let range2 = Range::new(15, 15, 25, 25);
        let union = range1.union(&range2).unwrap();

        assert_eq!(union.start_line(), 10);
        assert_eq!(union.start_column(), 10);
        assert_eq!(union.end_line(), 25);
        assert_eq!(union.end_column(), 25);
    }
}
