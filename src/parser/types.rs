use std::{
    cmp::PartialEq,
    collections::HashMap,
    fmt::{self, Formatter, Display},
    hash::{Hash, Hasher},
    string::String,
};

use serde::{Deserialize, Serialize};

use crate::utility::position::Range;
use crate::utility::util;
use self::util::*;

// Enums
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, Serialize, Deserialize)]
pub enum Operator {
    Equals = 0,
    GreaterThan = 1,
    LessThan = 2,
    GreaterThanOrEqual = 3,
    LessThanOrEqual = 4,
    NotEqual = 5,
    EqualEqual = 6,
    QuestionEqual = 7,
}

fn operator_to_string(op: Operator) -> &'static str {
    match op {
        Operator::Equals => "=",
        Operator::GreaterThan => ">",
        Operator::LessThan => "<",
        Operator::GreaterThanOrEqual => ">=",
        Operator::LessThanOrEqual => "<=",
        Operator::NotEqual => "!=",
        Operator::EqualEqual => "==",
        Operator::QuestionEqual => "?="
    }
}

// Key struct
#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Key(String);

impl Key {
    pub fn new(key: String) -> Self {
        Key(key)
    }
    
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Value enum
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    String(StringTokens),
    QString(StringTokens),
    Float(f64),
    Int(i32),
    Bool(bool),
    Clause(Vec<Statement>),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Clause(statements) => write!(f, "{{ {:?} }}", statements),
            Value::QString(s) => write!(f, "\"{:?}\"", s),  // pretty printing will do for now (TODO: fix this if needed)
            Value::String(s) => write!(f, "{:?}", s),
            Value::Bool(b) => write!(f, "{}", if *b { "yes" } else { "no" }),
            Value::Float(flt) => write!(f, "{}", flt),
            Value::Int(i) => write!(f, "{}", i),
        }
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::String(s) => s.hash(state),
            Value::QString(s) => s.hash(state),
            Value::Float(f) => f.to_bits().hash(state),  // Use bitwise representation of f64
            Value::Int(i) => i.hash(state),
            Value::Bool(b) => b.hash(state),
            Value::Clause(statements) => statements.hash(state),
        }
    }
}

impl Value {

    pub fn to_string(&self, string_manager: &StringResourceManager) -> String {
        match self {
            Value::Clause(statements) => {
                // Convert each Statement to its string representation
                let statement_str: Vec<String> = statements
                    .iter()
                    .map(|stmt| match stmt {
                        Statement::Comment(_, comment) => format!("# {}", comment),
                        Statement::KeyValue(key_value) => format!("{:?}", key_value), // Adjust as needed
                        Statement::Value(_, value) => value.to_string(string_manager),
                    })
                    .collect();
                format!("{{ {} }}", statement_str.join(", "))
            }
            Value::QString(tokens) => {
                format!("\"{}\"", string_manager.get_string_for_ids(tokens).unwrap_or_default())
            }
            Value::String(tokens) => {
                string_manager.get_string_for_ids(tokens).unwrap_or_default()
            }
            Value::Bool(b) => {
                if *b { "yes".to_string() } else { "no".to_string() }
            }
            Value::Float(f) => {
                format!("{}", f)
            }
            Value::Int(i) => {
                format!("{}", i)
            }
        }
    }
    pub fn to_raw_string(&self, string_manager: &StringResourceManager) -> String {
        match self {
            Value::Clause(statements) => {
                // Convert each Statement to its raw string representation
                let statement_str: Vec<String> = statements
                    .iter()
                    .map(|stmt| match stmt {
                        Statement::Comment(_, comment) => comment.clone(),
                        Statement::KeyValue(key_value) => format!("{:?}", key_value), // Adjust as needed
                        Statement::Value(_, value) => value.to_raw_string(string_manager),
                    })
                    .collect();
                format!("{{ {} }}", statement_str.join(", "))
            }
            Value::QString(tokens) | Value::String(tokens) => {
                string_manager.get_string_for_ids(tokens).unwrap_or_else(|| "".to_string())
            }
            Value::Bool(b) => {
                if *b { "yes".to_string() } else { "no".to_string() }
            }
            Value::Float(f) => {
                format!("{}", f)
            }
            Value::Int(i) => {
                format!("{}", i)
            }
        }
    }

    /// Converts the Value into its tokenized ID, or interns it if necessary.
    pub fn to_string_id(&self, string_manager: &StringResourceManager) -> StringTokens {
        match self {
            Value::String(tokens) | Value::QString(tokens) => tokens.clone(),
            _ => string_manager.intern_identifier_token(&self.to_string(string_manager)),
        }
    }
}

// KeyValueItem struct
#[derive(Debug, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct KeyValueItem {
    pub(crate) key: Key,
    pub(crate) value: Value,
    pub(crate) operator: Operator,
}

impl Display for KeyValueItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.key, operator_to_string(self.operator), self.value)
    }
}

// PosKeyValue struct
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PosKeyValue {
    pub(crate) range: Range,
    pub(crate) kv_item: KeyValueItem,
}

impl PartialEq for PosKeyValue {
    fn eq(&self, other: &Self) -> bool {
        self.kv_item == other.kv_item
    }
}

impl fmt::Display for PosKeyValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kv_item)
    }
}

impl Eq for PosKeyValue {}

impl std::hash::Hash for PosKeyValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kv_item.hash(state);
    }
}

// Statement enum
#[derive(Debug, Serialize, Deserialize)]
pub enum Statement {
    Comment(Range, String),  // range and comment string
    KeyValue(PosKeyValue),
    Value(Range, Value),
}

// Implement PartialEq manually to mirror F# equality:
impl PartialEq for Statement {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Statement::Comment(r1, s1), Statement::Comment(r2, s2)) => r1 == r2 && s1 == s2,
            (Statement::KeyValue(kv1), Statement::KeyValue(kv2)) => kv1 == kv2,
            (Statement::Value(r1, v1), Statement::Value(r2, v2)) => r1 == r2 && v1 == v2,
            _ => false,
        }
    }
}

impl Eq for Statement {}

// Implement Hash to mirror the F# GetHashCode logic.
// Note: For Comment and Value we only hash the comment string and value respectively.
impl Hash for Statement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Statement::Comment(_, comment) => {
                comment.hash(state);
            }
            Statement::KeyValue(pos_kv) => {
                pos_kv.hash(state);
            }
            Statement::Value(_, value) => {
                value.hash(state);
            }
        }
    }
}

// Optionally, implement Display to get a string representation of a Statement.
impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Comment(range, comment) => {
                write!(f, "Comment at {:?}: {}", range, comment)
            }
            Statement::KeyValue(pos_kv) => write!(f, "KeyValue: {}", pos_kv),
            Statement::Value(range, value) => write!(f, "Value at {:?}: {}", range, value),
        }
    }
}

// ParsedFile struct
#[derive(Debug, PartialEq)]
pub(crate) struct ParsedFile {
    pub(crate) statements: Vec<Statement>,
}

// APIs
type ParseFile = fn(String) -> Result<ParsedFile, ()>;
type ParseString = fn(String, String) -> Result<ParsedFile, ()>;
type PrettyPrintFile = fn(ParsedFile) -> String;
type PrettyPrintStatements = fn(Vec<Statement>) -> String;
type PrettyPrintStatement = fn(Statement) -> String;
type PrettyPrintFileResult = fn(Result<ParsedFile, ()>) -> String;

struct ParserAPI {
    parse_file: ParseFile,
    parse_string: ParseString,
}

struct PrinterAPI {
    pretty_print_file: PrettyPrintFile,
    pretty_print_statements: PrettyPrintStatements,
    pretty_print_statement: PrettyPrintStatement,
    pretty_print_file_result: PrettyPrintFileResult,
}