use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const COLUMN_BIT_COUNT: i32 = 11;
const LINE_BIT_COUNT: i32 = 21;
const POS_BIT_COUNT: i32 = LINE_BIT_COUNT + COLUMN_BIT_COUNT;
const POS_COLUMN_MASK: i32 = (1 << COLUMN_BIT_COUNT) - 1;
const LINE_COLUMN_MASK: i32 = ((1 << LINE_BIT_COUNT) - 1) << COLUMN_BIT_COUNT;

// Masks for range encoding
const START_LINE_SHIFT: i64 = 0;
const START_COLUMN_SHIFT: i64 = 21;
const HEIGHT_SHIFT: i64 = 32;
const END_COLUMN_SHIFT: i64 = 52;
const IS_SYNTHETIC_SHIFT: i64 = 63;

const START_LINE_MASK: i64 = ((1 << LINE_BIT_COUNT) - 1) << START_LINE_SHIFT;
const START_COLUMN_MASK: i64 = ((1 << COLUMN_BIT_COUNT) - 1) << START_COLUMN_SHIFT;
const HEIGHT_MASK: i64 = ((1 << 20) - 1) << HEIGHT_SHIFT;
const END_COLUMN_MASK: i64 = ((1 << COLUMN_BIT_COUNT) - 1) << END_COLUMN_SHIFT;
const IS_SYNTHETIC_MASK: i64 = 1 << IS_SYNTHETIC_SHIFT;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Pos {
    code: i32,
}

impl Pos {
    pub fn new(line: i32, col: i32) -> Self {
        let line = line.max(0);
        let col = col.max(0);
        let code = (col & POS_COLUMN_MASK) | ((line << COLUMN_BIT_COUNT) & LINE_COLUMN_MASK);
        Pos { code }
    }

    pub fn line(&self) -> i32 {
        self.code >> COLUMN_BIT_COUNT
    }

    pub fn column(&self) -> i32 {
        self.code & POS_COLUMN_MASK
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Range {
    code: i64,
    pub file_index: i16,
}

impl Range {
    pub fn new(file_index: i16, start: Pos, end: Pos) -> Self {
        let code = ((start.line() as i64) << START_LINE_SHIFT)
            | ((start.column() as i64) << START_COLUMN_SHIFT)
            | (((end.line() - start.line()) as i64) << HEIGHT_SHIFT)
            | ((end.column() as i64) << END_COLUMN_SHIFT);
        Range { code, file_index }
    }

    pub fn start_line(&self) -> i32 {
        ((self.code & START_LINE_MASK) >> START_LINE_SHIFT) as i32
    }

    pub fn start_column(&self) -> i32 {
        ((self.code & START_COLUMN_MASK) >> START_COLUMN_SHIFT) as i32
    }

    pub fn end_line(&self) -> i32 {
        self.start_line() + ((self.code & HEIGHT_MASK) >> HEIGHT_SHIFT) as i32
    }

    pub fn end_column(&self) -> i32 {
        ((self.code & END_COLUMN_MASK) >> END_COLUMN_SHIFT) as i32
    }

    pub fn is_synthetic(&self) -> bool {
        (self.code & IS_SYNTHETIC_MASK) != 0
    }

    pub fn contains_pos(&self, pos: &Pos) -> bool {
        (self.start_line() < pos.line()
            || (self.start_line() == pos.line() && self.start_column() <= pos.column()))
            && (self.end_line() > pos.line()
                || (self.end_line() == pos.line() && self.end_column() >= pos.column()))
    }

    pub fn union_ranges(m1: Range, m2: Range) -> Range {
        if m1.file_index != m2.file_index {
            m2
        } else {
            let b = if pos_gt(m2.start, m1.start) { m2.start } else { m1.start };
            let e = if pos_gt(m1.end, m2.end) { m1.end } else { m2.end };
            mk_range(m1.file_index, b, e, false)
        }
    }
}

pub(crate) fn memoize<F, A, R>(func: F) -> impl FnMut(A) -> R
where
    F: Fn(A) -> R,
    A: Eq + std::hash::Hash + Clone,
    R: Clone,
{
    let mut cache: HashMap<A, R> = HashMap::new();
    move |arg: A| {
        if let Some(result) = cache.get(&arg) {
            return result.clone();
        }
        let result = func(arg.clone());
        cache.insert(arg, result.clone());
        result
    }
}

pub(crate) fn mk_pos(line: i32, col: i32) -> Pos {
    Pos::new(line, col)
}

pub(crate) fn mk_range(file: &str, start: Pos, end: Pos) -> Range {
    let file_index = file_index_of_file(file);
    Range::new(file_index, start, end)
}

pub fn pos_gt(p1: Pos, p2: Pos) -> bool {
    p1.line > p2.line || (p1.line == p2.line && p1.column > p2.column)
}

pub fn pos_eq(p1: Pos, p2: Pos) -> bool {
    p1.line == p2.line && p1.column == p2.column
}

pub fn pos_geq(p1: Pos, p2: Pos) -> bool {
    pos_eq(p1, p2) || pos_gt(p1, p2)
}

// Example function for path normalization, similar to mkRangePath
pub(crate) fn normalize_path(path: &str) -> String {
    if std::path::Path::new(path).is_absolute() {
        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_string().into()).to_str().unwrap().to_string()
    } else {
        path.to_string()
    }
}

struct FileIndexTable {
    index_to_file: Vec<String>,
    file_to_index: HashMap<String, i16>,
}

impl FileIndexTable {
    pub fn new() -> Self {
        FileIndexTable {
            index_to_file: Vec::new(),
            file_to_index: HashMap::new(),
        }
    }

    pub fn file_to_index(&mut self, file: &str) -> i16 {
        if let Some(index) = self.file_to_index.get(file) {
            return *index;
        }
        let index = self.index_to_file.len() as i16;
        self.index_to_file.push(file.to_string());
        self.file_to_index.insert(file.to_string(), index);
        index
    }

    pub fn index_to_file(&self, index: i16) -> &str {
        if (index as usize) < self.index_to_file.len() {
            &self.index_to_file[index as usize]
        } else {
            panic!("Invalid file index");
        }
    }
}

// Global mutable state for file index table
lazy_static::lazy_static! {
    static ref FILE_INDEX_TABLE: Arc<Mutex<FileIndexTable>> = Arc::new(Mutex::new(FileIndexTable::new()));
}

pub(crate) fn file_index_of_file(file: &str) -> i16 {
    let mut table = FILE_INDEX_TABLE.lock().unwrap();
    table.file_to_index(file)
}

pub(crate) fn file_of_file_index(index: i16) -> String {
    let table = FILE_INDEX_TABLE.lock().unwrap();
    table.index_to_file(index).to_string()
}

