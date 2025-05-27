use log::{debug, info, warn};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while1, take_while_m_n},
    character::complete::{char, char as nom_char, digit1, multispace0, multispace1, not_line_ending, satisfy},
    combinator::{map, not, opt, peek, recognize},
    error::{context, ParseError},
    multi::{many0, many1},
    sequence::{delimited, preceded, terminated},
    IResult as NomResult, Parser,
};
use nom_locate::LocatedSpan;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, ops::Range};

use crate::utility::error::{Error, ErrorContext};
use crate::utility::util;
use crate::parser::types;

use self::util::*;
use self::types::*;

// Utility functions

/// A wrapper function to add logging to a parser.
pub fn with_logging<'a, F, O, E>(
    mut parser: F,
    label: &'static str,
) -> impl FnMut(Span<'a>) -> IResult<'a, O>
where
    F: FnMut(&'a str) -> IResult<'a, O>,
    E: ParseError<&'a str>,
{
    move |input: Span<'a>| {
        info!("Entering parser: {}", label);
        let result = parser(&input);
        match &result {
            Ok(_) => info!("Leaving parser: {} (Success)", label),
            Err(_) => info!("Leaving parser: {} (Error)", label),
        }
        result
    }
}


pub(crate) type Span<'a> = LocatedSpan<&'a str, State<'a>>;
type IResult<'a, O> = NomResult<Span<'a>, O>;

#[derive(Copy, Clone, Debug)]
pub struct State<'a>(pub &'a ErrorContext);

impl State<'_> {
    /// Pushes an error onto the errors stack while still allowing parsing to continue.
    pub fn report_error(&self, error: Error) {
        self.0.add_error(error);
    }
}

/// Evaluate `parser` and wrap the result in a `Some(_)`. Otherwise,
/// emit the  provided `error_msg` and return a `None` while allowing
/// parsing to continue.
fn expect<'a, F, E, T>(mut parser: F, error_msg: E) -> impl FnMut(Span<'a>) -> IResult<'a, Option<T>>
where
    F: FnMut(Span<'a>) -> IResult<'a, T>,
    E: ToString,
{
    move |input| match parser(input) {
        Ok((remaining, out)) => Ok((remaining, Some(out))),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            let err = Error(input.to_range(), error_msg.to_string());
            input.extra.report_error(err);
            Ok((input, None)) // Parsing failed, but keep going.
        }
        Err(err) => Err(err),
    }
}

pub fn expect_with_error<'a, F, T>(
    mut parser: F,
    msg: &'static str,
) -> impl FnMut(Span<'a>) -> IResult<'a, T>
where
    F: FnMut(Span<'a>) -> IResult<'a, T>,
{
    move |input: Span<'a>| match parser(input) {
        Ok((next, out)) => Ok((next, out)),
        Err(nom::Err::Error(_)) | Err(nom::Err::Failure(_)) => {
            input.extra.0.errors.borrow_mut().push(Error(
                input.to_range(),
                msg.to_string(),
            ));
            Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Alt)))
        }
        Err(e) => Err(e),
    }
}

trait ToRange {
    fn to_range(&self) -> Range<usize>;
}
impl ToRange for Span<'_> {
    fn to_range(&self) -> Range<usize> {
        let start = self.location_offset();
        let end = start + self.fragment().len();
        start..end
    }
}

pub fn truncate_input(input: &str, max_lines: usize) -> String {
    input
        .lines()
        .take(max_lines)
        .collect::<Vec<&str>>()
        .join("\n")
}

// Sets of chars
// =======
const WHITESPACE_TEXT_CHARS: &str = " \t\r\n";

const NORSE_CHARS: &[char] = &['ö', 'ð', 'æ', 'ó', 'ä', 'Þ', 'Å', 'Ö'];

const ID_CHAR_ARRAY: &[char] = &[
    '_', ':', '@', '.', '\"', '-', '\'', '[', ']', '!', '<', '>', '$', '^', '&', '|', util::MAGIC_CHAR,
];

// Characters that can be part of a value
// pretty sure semicolon is not a value char and is meant to be dropped
// but it is used in the F# code so we will keep it for now
const VALUE_CHAR_ARRAY: &[char] = &[
    '_', '.', '-', ':', ';', '\'', '[', ']', '@', '\'', '+', '`', '%', '/', '!', ',', '<', '>',
    '?', '$', 'š', 'Š', '’', '|', '^', '*', '&', '“', '”', util::MAGIC_CHAR,
];

const QUOTE_CHAR: char = '"';

fn ws<'a, F, O>(mut parser: F) -> impl FnMut(Span<'a>) -> IResult<'a, O>
where
    F: FnMut(Span<'a>) -> IResult<'a, O>,
{
    move |input| delimited(multispace0, &mut parser, multispace0).parse(input)
}

fn is_quote_char(c: char) -> bool {
    c == QUOTE_CHAR
}

fn is_any_of_id_char(c: char) -> bool {
    ID_CHAR_ARRAY.contains(&c)
}

fn is_any_value_char(c: char) -> bool {
    VALUE_CHAR_ARRAY.contains(&c)
}

fn is_id_char(c: char) -> bool {
    c.is_alphabetic() || c.is_ascii_digit() || is_any_of_id_char(c)
}

fn is_value_char(c: char) -> bool {
    (c.is_alphabetic() || c.is_ascii_digit() || is_any_value_char(c)) && c != '{' && c != '}'
}

// Match a specific string, followed by optional whitespace
fn str_parser(s: &'static str) -> impl FnMut(Span) -> IResult<&str> {
    move |input: Span| terminated(tag(s), multispace0).parse(input).map(|(next_input, _)| (next_input, s))
}

// Skip a specific string, followed by optional whitespace
fn str_skip(s: &'static str) -> impl FnMut(Span) -> IResult<()> {
    move |input: Span| terminated(tag(s), multispace0).parse(input).map(|(next_input, _)| (next_input, ()))
}

// Match a specific character, followed by optional whitespace
fn ch_parser(c: char) -> impl FnMut(Span) -> IResult<char> {
    move |input: Span| terminated(nom_char(c), multispace0).parse(input)
}

// Skip a specific character, followed by optional whitespace
fn ch_skip(c: char) -> impl FnMut(Span) -> IResult<()> {
    move |input: Span| terminated(nom_char(c), multispace0).parse(input).map(|(next_input, _)| (next_input, ()))
}

/// Matches one or more characters that are NOT '\' or '"'
fn quoted_char_snippet(input: Span) -> IResult<&str> {
    //debug!("THIS IS QUOTED_CHAR_SNIPPET. Parsing quoted_char_snippet: {:?}", truncate_input(&input, 2));
    let (input, chars) = take_while1(|c: char| c != '\\' && c != '"')(input)?;
    Ok((input, chars.fragment()))
}

/// Matches an escaped sequence: either `\"` or `\`
pub fn escaped_char(input: Span) -> IResult<'_, &str> {
    //debug!("THIS IS ESCAPED_CHAR. Parsing escaped_char: {:?}", truncate_input(&input, 2));
    map(
        alt((
            tag("\\\""),
            tag("\\"),
        )),
        |s: Span| *s.fragment(), // Extract &str from Span
    ).parse(input)
}

fn metaprogramming_char_snippet(input: Span) -> IResult<&str> {
    let (input, chars) = is_not("]\\")(input)?;
    Ok((input, chars.fragment()))
}

// A simple version of between_l that uses nom::error::Error
pub fn between_l<'a, F, G, H, O1, O2, O3>(
    mut popen: F,
    mut pclose: G,
    mut p: H,
    label: &'static str,
) -> impl FnMut(Span<'a>) -> IResult<'a, O2>
where
    F: FnMut(Span<'a>) -> IResult<'a, O1>,
    G: FnMut(Span<'a>) -> IResult<'a, O3>,
    H: FnMut(Span<'a>) -> IResult<'a, O2>,
{
    move |input: Span| {
        //debug!("Parsing between_l ({}): {:?}", label, truncate_input(&input, 2));

        // Match the opening delimiter
        let (input, _) = popen(input)?;

        // Parse the inner content
        let (remaining, output_inner) = p(input)?;

        // Attempt to match the closing delimiter
        match pclose(remaining) {
            Ok((remaining, _)) => {
                //debug!("Successfully matched closing delimiter for {}", label);
                Ok((remaining, output_inner))
            }
            Err(_) => {
                // Check if the remaining input is EOF
                match nom::combinator::eof(remaining) {
                    Ok((remaining, _)) => {
                        warn!(
                            "Unclosed top-level bracket detected at EOF for {}. Treating it as implicitly closed.",
                            label
                        );
                        Ok((remaining, output_inner)) // Treat as implicitly closed
                    }
                    Err(e) => Err(e), // Propagate other errors
                }
            }
        }
    }
}

/// A clause parser that expects the inner content to be between `{` and `}`.
fn clause<'a, O, F>(inner: F) -> impl FnMut(Span<'a>) -> IResult<'a , O>
where
    F: FnMut(Span<'a>) -> IResult<'a, O>,
{
    between_l(nom_char('{'), nom_char('}'), inner, "clause")
}

// Use `LocatedSpan` for input type to track positions
//type Span<'a> = LocatedSpan<&'a str>;

/// Get the range from start and end spans
pub fn get_range<'a>(start: Span<'a>, end: Span<'a>) -> Range<usize> {
    let start_line = start.location_line() as i32;
    let start_column = start.get_column() as i32;
    let end_line = end.location_line() as i32;
    let end_column = end.get_column() as i32;

    assert!(
        start_line >= 0 && start_column >= 0 && end_line >= 0 && end_column >= 0,
        "Line and column values must be non-negative."
    );
    assert!(
        start_line < (1 << 21) && start_column < (1 << 11),
        "Start line/column exceeds bit constraints."
    );
    assert!(
        end_line < (1 << 21) && end_column < (1 << 11),
        "End line/column exceeds bit constraints."
    );

    Range {
        start: (start_line << 11 | start_column) as usize,
        end: (end_line << 11 | end_column) as usize,
    }
}

fn parse_with_position<'a, O, F>(mut parser: F) -> impl FnMut(Span<'a>) -> IResult<'a, (Range<usize>, O)>
where
    F: FnMut(Span<'a>) -> IResult<'a, O>,
{
    move |input: Span<'a>| {
        let start = input;
        let (next, output) = parser(input)?;
        let range = get_range(start, next);
        Ok((next, (range, output)))
    }
}

fn operator(input: Span<'_>) -> IResult<'_, Operator> {
    //debug!("THIS IS OPERATOR. Parsing operator: {:?}", truncate_input(input.fragment(), 2));
    let (i, op) = alt((
        map(ws(tag("<=")), |_| Operator::LessThanOrEqual),
        map(ws(tag(">=")), |_| Operator::GreaterThanOrEqual),
        map(ws(tag("!=")), |_| Operator::NotEqual),
        map(ws(tag("==")), |_| Operator::EqualEqual),
        map(ws(tag("?=")), |_| Operator::QuestionEqual),
        map(ws(tag("<")), |_| Operator::LessThan),
        map(ws(tag(">")), |_| Operator::GreaterThan),
        map(ws(tag("=")), |_| Operator::Equals),
    )).parse(input)?;
    Ok((i, op))
}

fn operator_lookahead(input: Span) -> IResult<Span> {
    peek(alt((
        ws(tag("=")),
        ws(tag(">")),
        ws(tag("<")),
        ws(tag("!")),
        ws(tag("?=")),
    ))).parse(input)
}

/// Parses a comment and captures its positional metadata.
pub fn comment(input: Span) -> IResult<(Range<usize>, String)> {
    let start = input;
    let (input, content) = terminated(
        preceded(nom_char('#'), not_line_ending),
        multispace0,
    ).parse(input)?;
    let range = start.to_range();
    Ok((input, (range, content.trim().to_string())))
}

/// Key parser that returns a `Key` struct
fn key(input: Span) -> IResult<Key> {
    let key_parser = map(
        // Use take_while1 to require at least one character that satisfies is_id_char
        terminated(take_while1(is_id_char), multispace0),
        |s: Span| Key::new(s.to_string()),
    );
    let mut parser = preceded(multispace0, key_parser); // Skip leading whitespace
    //debug!("THIS IS KEY. Parsing key: {:?}", truncate_input(&input, 2));
    
    //debug!("THIS IS KEY. Parsed key: {:?}", res);
    parser.parse(input)
    
}

// Key parser that matches a quoted key
fn key_q(input: Span) -> IResult<Key> {
    //debug!("Parsing key_q: {:?}", truncate_input(&input, 2));
    map(
        quoted_string, // Use the `quoted_string` parser
        |s: String| Key::new(s), // Wrap the parsed string in a `Key` struct
    ).parse(input)
}

fn value_s_old<'a>(
    input: Span<'a>,
    string_manager: &'a StringResourceManager,
) -> IResult<'a, Value> {
    //debug!("THIS IS VALUE_S. Parsing value_s: {:?}", truncate_input(&input, 2));

    // Check for a `"` at the beginning of the string and log a warning
    // we'll make this more robust later
    if peek(tag::<_, _, nom::error::Error<&str>>("\"")).parse(&input).is_ok() {
        warn!(
            "Detected a `\"` at the beginning of the string in value_s: {:?}",
            truncate_input(&input, 2)
        );
    }
    context(
        "string",
        map(
            // Use `delimited` to discard the `"` at the beginning and end of the because if its here it escaped from value_q
            delimited(opt(tag("\"")),take_while1(is_value_char),opt(tag("\""))),
            |s: Span| Value::String(string_manager.intern_identifier_token(&s)),
        ),
    ).parse(input)
}

fn value_s<'a>(
    input: Span<'a>,
    string_manager: &'a StringResourceManager,
) -> IResult<'a, Value> {
    //debug!("THIS IS VALUE_S. Parsing value_s: {:?}", truncate_input(&input, 2));

    if peek(tag::<_, _, nom::error::Error<Span>>("\"")).parse(input).is_ok() {
        warn!(
            "Detected a `\"` at the beginning of the string in value_s: {:?}",
            truncate_input(&input, 2)
        );
    }

    context(
        "string value",
        map(
            delimited(
                opt(tag("\"")),
                recognize(alt((
                    take_while1(is_value_char),
                    map(
                        take_while1(|c: char| !c.is_whitespace() && c != '#' && c != '\n' && c != '\r' && c != '\"' && c != '{' && c != '}'),
                        |s: Span<'a>| {
                            // Pull out the error context from the span's state
                            let error_ctx = s.extra.0;
                            let start = s.location_offset();
                            let end = start + s.fragment().len();
                            let msg = format!("Unexpected character in string: '{}'", s.fragment());
                            error_ctx.add_error(Error(start..end, msg));
                            s
                        }
                    ),
                ))),
                opt(tag("\""))
            ),
            |s: Span<'a>| Value::String(string_manager.intern_identifier_token(&s)),
        ),
    ).parse(input)
}

fn value_i(input: Span) -> IResult<Value> {
    map(take_while_m_n(1, 10, |c: char| c.is_ascii_digit()), |s: Span| Value::Int(s.fragment().parse::<i32>().unwrap())).parse(input)
}
// A parser for floating point numbers (e.g., "123.456")
fn value_f(input: Span) -> IResult<Value> {
    map(
        recognize(
            (digit1, char('.'), digit1)
        ),
        |s: Span| Value::Float(s.parse::<f64>().unwrap())
    ).parse(input)
}

fn value_b_yes(input: Span) -> IResult<Value> {
    map(
        preceded(
            tag("yes"),
            peek(not(satisfy(|c| is_value_char(c) && c != '}'))), // Allow `yes` to be followed by `}` or whitespace
        ),
        |_| Value::Bool(true), // Return `Value::Bool(true)`
    ).parse(input)
}

fn value_b_no(input: Span) -> IResult<Value> {
    map(
        preceded(
            tag("no"),
            peek(not(satisfy(|c| is_value_char(c) && c != '}'))), // Allow `no` to be followed by `}` or whitespace
        ),
        |_| Value::Bool(false), // Return `Value::Bool(false)`
    ).parse(input)
}

// Match a quoted string (with escape sequences) with additional checks
// this is a roundabout way to ensure that quotes are opened and closed properly
// this is a bit of a hack and likely still has edge cases
// it may consider a string to be valid even if it is not
// or vice versa
// better then the alternative of not enforcing string validity at all
fn quoted_string(input: Span) -> IResult<String> {
    //debug!("Parsing quoted_string: {:?}", truncate_input(&input, 2));
    let mut parser = delimited(
            char('"'), // Match the opening quote
            map(
                many0(alt((
                    quoted_char_snippet, // Match characters that are not `\` or `"`
                    escaped_char,        // Match escaped sequences like `\"` or `\\`
                ))),
                |parts: Vec<&str>| parts.concat(), // Combine all parts into a single string
            ),
            terminated(
                char('"'), // Match the closing quote
                peek(alt((
                    multispace1, // Allow whitespace
                    operator_lookahead, // Allow operators - using lookahead is probably redundant 
                    tag("}"), // Allow closing brace
                    tag("\""), // Allow next quote
                    tag("#"), // Allow comment
                    tag(","), // Allow commas 
                    nom::combinator::eof, // Allow end of file

                    tag(";"), // afaik we want to drop semicolons but we'll pass the buck

                ))),
            ),
        );

        parser.parse(input)
}

fn value_q<'a>(
    input: Span<'a>,
    string_manager: &StringResourceManager,
) -> IResult<'a, Value> {
    //debug!("THIS IS VALUE_Q. Parsing value_q: {:?}", truncate_input(&input, 2));
    map(
        quoted_string, // Parse the quoted string
        |s: String| {
            let token = string_manager.intern_identifier_token(&s); // Intern the string
            Value::QString(token) // Return it as a Value::QString
        },
    ).parse(input)
}

fn hsv3(input: Span) -> IResult<'_, Value> {
    map(
        (
            terminated(terminated(parse_with_position(value_f), multispace0), multispace0),
            terminated(parse_with_position(value_f), multispace0),
            terminated(parse_with_position(value_f), multispace0),
    ),
        |(a, b, c)| {
            Value::Clause(vec![
                Statement::Value(a.0, a.1),
                Statement::Value(b.0, b.1),
                Statement::Value(c.0, c.1),
            ])
        }
    ).parse(input)
}

fn hsv4(input: Span) -> IResult<Value> {
    map(
        (
            // First value: apply parse_with_position(value_f), then consume ws twice.
            terminated(terminated(parse_with_position(value_f), multispace0), multispace0),
            // Second value: apply parse_with_position(value_f), then consume ws.
            terminated(parse_with_position(value_f), multispace0),
            // Third value: apply parse_with_position(value_f), then consume ws.
            terminated(parse_with_position(value_f), multispace0),
            // Fourth val
            terminated(parse_with_position(value_f), multispace0),
        ),
        |(a, b, c, d)| {
            Value::Clause(vec![
                Statement::Value(a.0, a.1),
                Statement::Value(b.0, b.1),
                Statement::Value(c.0, c.1),
                Statement::Value(d.0, d.1)
            ])
        }
    ).parse(input)
}

/// Parser for hsvI (a clause with 3 or 4 float values).
fn hsv_i(input: Span) -> IResult<Value> {
    map(
        (
            // Each value: run parse_with_position(value_f) then consume whitespace.
            terminated(terminated(parse_with_position(value_f), multispace0), multispace0),
            terminated(parse_with_position(value_f), multispace0),
            terminated(parse_with_position(value_f), multispace0),
            // Optional fourth value.
            opt(terminated(parse_with_position(value_f), multispace0)),
        ),
        |(a, b, c, d)| {
            match d {
                Some(d_val) => Value::Clause(vec![
                    Statement::Value(a.0, a.1),
                    Statement::Value(b.0, b.1),
                    Statement::Value(c.0, c.1),
                    Statement::Value(d_val.0, d_val.1),
                ]),
                None => Value::Clause(vec![
                    Statement::Value(a.0, a.1),
                    Statement::Value(b.0, b.1),
                    Statement::Value(c.0, c.1),
                ]),
            }
        }
    ).parse(input)
}

/// Parser for hsv:
///   strSkip "hsv" >>. opt (strSkip "360") >>. hsvI .>> ws
fn hsv(input: Span) -> IResult<Value> {
    let (input, _) = str_skip("hsv")(input)?;
    let (input, _) = opt(str_skip("360")).parse(input)?;
    terminated(hsv_i, multispace0).parse(input)
}

/// Parser for hsvC:
///   strSkip "HSV" >>. hsvI .>> ws
fn hsv_c(input: Span) -> IResult<Value> {
    let (input, _) = str_skip("HSV")(input)?;
    terminated(hsv_i, multispace0).parse(input)
}

/// Parser for rgbI (a clause with 3 or 4 integer values).
fn rgb_i(input: Span) -> IResult<Value> {
    map(
        (
            terminated(terminated(parse_with_position(value_i), multispace0), multispace0),
            terminated(parse_with_position(value_i), multispace0),
            terminated(parse_with_position(value_i), multispace0),
            opt(terminated(parse_with_position(value_i), multispace0)),
        ),
        |(a, b, c, d)| {
            match d {
                Some(d_val) => Value::Clause(vec![
                    Statement::Value(a.0, a.1),
                    Statement::Value(b.0, b.1),
                    Statement::Value(c.0, c.1),
                    Statement::Value(d_val.0, d_val.1),
                ]),
                None => Value::Clause(vec![
                    Statement::Value(a.0, a.1),
                    Statement::Value(b.0, b.1),
                    Statement::Value(c.0, c.1),
                ]),
            }
        }
    ).parse(input)
}

/// Parser for rgb3 (a clause with exactly 3 integer values).
fn rgb3(input: Span) -> IResult<Value> {
    map(
        (
            terminated(terminated(parse_with_position(value_i), multispace0), multispace0),
            terminated(parse_with_position(value_i), multispace0),
            terminated(parse_with_position(value_i), multispace0),
        ),
        |(a, b, c)| {
            Value::Clause(vec![
                Statement::Value(a.0, a.1),
                Statement::Value(b.0, b.1),
                Statement::Value(c.0, c.1),
            ])
        }
    ).parse(input)
}

/// Parser for rgb4 (a clause with exactly 4 integer values).
fn rgb4(input: Span) -> IResult<Value> {
    map(
        (
            terminated(terminated(parse_with_position(value_i), multispace0), multispace0),
            terminated(parse_with_position(value_i), multispace0),
            terminated(parse_with_position(value_i), multispace0),
            terminated(parse_with_position(value_i), multispace0),
        ),
        |(a, b, c, d)| {
            Value::Clause(vec![
                Statement::Value(a.0, a.1),
                Statement::Value(b.0, b.1),
                Statement::Value(c.0, c.1),
                Statement::Value(d.0, d.1),
            ])
        }
    ).parse(input)
}

/// Parser for rgb:
///   strSkip "rgb" >>. rgbI .>> ws
fn rgb(input: Span) -> IResult<Value> {
    let (input, _) = str_skip("rgb")(input)?;
    terminated(rgb_i, multispace0).parse(input)
}

/// Parser for rgbC:
///   strSkip "RGB" >>. rgbI .>> ws
fn rgb_c(input: Span) -> IResult<Value> {
    let (input, _) = str_skip("RGB")(input)?;
    terminated(rgb_i, multispace0).parse(input)
}

fn metaprograming<'a>(
    input: Span<'a>,
    string_manager: &'a StringResourceManager,
) -> IResult<'a, Value> {
    map(
        (tag("@\\["), metaprogramming_char_snippet, char(']')),
        |(start, middle, end_char)| {
            // Concatenate the three pieces.
            let combined = format!("{}{}{}", start, middle, end_char);
            // Intern the concatenated string using the provided StringResourceManager.
            let token = string_manager.intern_identifier_token(&combined);
            // Wrap the interned token in the Value::String variant.
            Value::String(token)
        },
    ).parse(input)
}

fn leaf_value<'a>(input: Span<'a>, string_manager: &'a StringResourceManager) -> IResult<'a, (Range<usize>, Value)> {
    //debug!("Attempting to parse leaf_value from: {:?}", truncate_input(&input, 2));
    
    // Parse a value followed by trailing whitespace.
    let (input, val) = ws(|i| value(i, string_manager)).parse(input)?;
    
    // Lookahead: ensure the next token is NOT an operator.
    // If an operator is found, `not(peek(operator))` will fail without consuming input.
    let (input, _) = not(preceded(multispace0,peek(operator))).parse(input)?;
    
    let range = input.to_range(); // Get the range of the input
    
    //debug!("Returning leaf_value: {:?}", (range.clone(), &val));
    Ok((input, (range, val)))
}

fn value_block<'a>(input: Span<'a>, string_manager: &'a StringResourceManager) -> IResult<'a, Value> {
    let inner = alt((
        // Map leaf_value into a Statement::Value.
        map(
            |input| leaf_value(input, string_manager),
            |(range, val)| Statement::Value(range, val),
        ),
        // Map comment into a Statement::Comment.
        map(comment, |s| Statement::Comment(s.0, s.1)),
    ));
    let (input, stmts) = many0(inner).parse(input)?;
    Ok((input, Value::Clause(stmts)))
}

fn value_clause<'a>(
    input: Span<'a>,
    string_manager: &'a StringResourceManager,
) -> IResult<'a, Value> {
    //debug!("Parsing value_clause: {:?}", truncate_input(&input, 5));

    let mut parser = preceded(
        peek(tag("{")),
        |input| {
            clause(|input| {
                delimited(
                    multispace0,
                    many0(|input: Span<'a>| {
                        //debug!("Parsing nested statement in value_clause: {:?}", truncate_input(&input, 2));
                        statement(input, string_manager)
                    }),
                    multispace0,
                ).parse(input)
            })(input)
        },
    );

    parser.parse(input).map(|(remaining, stmts)| (remaining, Value::Clause(stmts)))
}
// ==================================================================
// valueCustom
// ==================================================================
//
// valueCustom inspects the first character (or prefix) of the input
// and then dispatches to the appropriate parser. This mirrors the F# code:
//
//   match stream.Peek() with
//   | '{' -> valueClause stream
//   | '"' -> valueQ stream
//   | x when isDigit x || x = '-' -> try valueI, else try valueF, else valueS
//   | _ -> match stream.PeekString 3, stream.PeekString 2 with
//          | "rgb", _ -> rgb stream
//          | "RGB", _ -> rgbC stream
//          | "hsv", _ -> hsv stream
//          | "HSV", _ -> hsvC stream
//          | "yes", _ -> valueBYes stream <|> valueS stream
//          | _, "no" -> valueBNo stream <|> valueS stream
//          | "@\\[", _ -> metaprograming stream
//          | _ -> valueS stream
//
// We assume the existence of the parsers: value_q, value_i, value_f, value_s,
// rgb, rgb_c, hsv, hsv_c, value_b_yes, value_b_no, and metaprograming.
fn value<'a>(
    input: Span<'a>,
    string_manager: &'a StringResourceManager,
) -> IResult<'a, Value> {
    if input.trim().is_empty() {
        //debug!("Input is empty, returning an error.");
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Eof)));
    }
    //debug!("Parsing value: {:?}", truncate_input(&input, 2));

    let mut parser = alt((
        // Use `peek` to check for specific starting characters or prefixes
        preceded(peek(tag("{")), |i| {
            //debug!("Matched peek for value_clause");
            value_clause(i, string_manager)}),
        preceded(peek(tag("\"")), |i| value_q(i, string_manager)),
        preceded(
            peek(satisfy(|c| c.is_ascii_digit() || c == '-')),
            alt((
                value_f,
                value_i,
                |i| value_s(i, string_manager),
            )),
        ),
        preceded(peek(tag("yes")), value_b_yes),
        preceded(peek(tag("no")), value_b_no),
        preceded(peek(tag("@\\")), |i| metaprograming(i, string_manager)),
        preceded(peek(tag("rgb")), rgb),
        preceded(peek(tag("RGB")), rgb_c),
        preceded(peek(tag("hsv")), hsv),
        preceded(peek(tag("HSV")), hsv_c),
        |i| value_s(i, string_manager),
    ));

    parser.parse(input)
}

// ==================================================================
// keyvalue parser
// ==================================================================
//
// keyvalue = pipe5 getPosition (keyQ <|> key) operator value (getPosition .>> ws)
//             (fun start id op value endp ->
//                  KeyValue(PosKeyValue(getRange start endp, KeyValueItem(id, value, op))))
fn keyvalue<'a>(
    input: Span<'a>,
    string_manager: &'a StringResourceManager,
) -> IResult<'a, Statement> {
    //debug!("Parsing keyvalue: {:?}", truncate_input(&input, 2));

    // Use `peek` to ensure the input starts with a valid key
    let (input, id) = preceded(peek(alt((key_q, key))), alt((key_q, key))).parse(input)?;
    //debug!("Parsed key: {:?}", id.to_string());

    let (input, op) = operator(input)?;
    //debug!("Parsed operator: {:?}", op);

        // Allow an optional comment and newline before the value clause
        // edge case handling
        // might be a terrible idea to implement this way
        let (input, _) = opt(terminated(comment, multispace0)).parse(input)?;

    // Try to parse the value, but if it fails, emit a specific error
    let value_result = value(input, string_manager);
    let (input, val) = match value_result {
        Ok((input, val)) => (input, val),
        Err(nom::Err::Error(_)) | Err(nom::Err::Failure(_)) => {
            // Add a specific error for missing value
            let msg = format!("Missing value for key: '{}'", id);
            input.extra.0.add_error(Error(input.to_range(), msg));
            return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Alt)));
        }
        Err(e) => return Err(e),
    };
    
    //debug!("Parsed value: {:?}", val.to_string(string_manager));

    let range = input.to_range();
    let kv_item = KeyValueItem {
        key: id,
        value: val,
        operator: op,
    };
    //debug!("Created KeyValueItem: {:?}", kv_item);

    Ok((input, Statement::KeyValue(PosKeyValue { range, kv_item })))
}

fn statement<'a>(
    input: Span<'a>,
    string_manager: &'a StringResourceManager,
) -> IResult<'a, Statement> {
    let (input, _) = multispace0(input)?;

    if input.fragment().is_empty() {
        //debug!("Input is empty, returning eof error to escape.");
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Eof)));
    }
    //debug!("Parsing statement: {:?}", truncate_input(&input, 2));

    let parse_comment = preceded(peek(tag("#")), map(comment, |s| {
        //debug!("Parsed comment: {:?}", s);
        Statement::Comment(s.0, s.1)
    }));

    let parse_leaf_value = preceded(
        peek(not(peek(operator_lookahead))),
        map(|i| leaf_value(i, string_manager), |(r, v)| Statement::Value(r, v)),
    );

    let parse_keyvalue = preceded(peek(alt((key_q, key))), |i| keyvalue(i, string_manager));
    

    terminated(alt((parse_comment, parse_leaf_value, parse_keyvalue)), multispace0).parse(input)
}

// ==================================================================
// Top–level parsers for a file
// ==================================================================
//
// alle = ws >>. many statement .>> eof |>> (fun f -> ParsedFile f)
// valuelist = many1 ((comment |>> Comment) <|> (leafValue |>> (fun (a, b) -> Value(a, b)))) .>> eof
// statementlist = many statement .>> eof
// all = ws >>. ((attempt valuelist) <|> statementlist)
//
// We define ParsedFile and Expr accordingly.

fn alle<'a>(input: Span<'a>, string_manager: &'a StringResourceManager) -> IResult<'a, ParsedFile> {
    multispace0(input)?; // Consume leading whitespace
    let (input, stmts) = many0(|i| statement(i, string_manager)).parse(input)?;
    let (input, _) = nom::combinator::eof(input)?;
    Ok((input, ParsedFile { statements: stmts }))
}

/// Parses one or more items, each of which is either a comment or a leaf value.
/// Each comment is mapped to Statement::Comment and each leaf value is mapped to Statement::Value.
/// The parser succeeds only if all input is consumed (via `eof`).
fn valuelist<'a>(input: Span<'a>, string_manager: &'a StringResourceManager) -> IResult<'a, Vec<Statement>> {
    let result = many1(
        delimited(
            multispace0,
            alt((
                map(comment, |(range, text)| Statement::Comment(range, text)),  
                map(
                    // skip leading space, ensure next non‐space isn't an operator, then parse leaf_value
                    preceded(
                        multispace0,
                        preceded(
                            not(peek(operator_lookahead)),
                            |i| leaf_value(i, string_manager)
                        ),
                    ),
                    |(r, v)| Statement::Value(r, v),
                ),
            )),
            multispace0,
        )
    ).parse(input);
    
    //debug!("valuelist parsed: {:?}", result);
    result
}


fn statementlist<'a>(input: Span<'a>, string_manager: &'a StringResourceManager) -> IResult<'a, Vec<Statement>> {
    let (input, _) = multispace0(input)?;

    if input.fragment().is_empty() {
        //debug!("Input is empty, checking for EOF.");
        return nom::combinator::eof(input).map(|(remaining, _)| (remaining, vec![]));
    }

    let (input, stmts) = many0(|i| statement(i, string_manager)).parse(input)?;
    //debug!(
    //    "Remaining input in statementlist (first 2 lines):\n{}",
    //    truncate_input(&input, 5)
    //);

    // Ensure all input is consumed
    let (input, _) = multispace0(input)?;
    let (input, _) = nom::combinator::eof.parse(input)?;
    Ok((input, stmts))
}

// For the top–level parser “all”, we assume that if valuelist fails we try statementlist.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Expr {
    Valuelist(Vec<Statement>),
    Statementlist(Vec<Statement>),
}

impl Expr {
    pub fn default() -> Self {
        Expr::Statementlist(vec![])
    }
}

fn bom(input: Span) -> IResult<()> {
    opt(tag("\u{feff}")).parse(input).map(|(next_input, _)| (next_input, ()))
}

pub(crate) fn all<'a>(input: Span<'a>, string_manager: &'a StringResourceManager) -> IResult<'a, Expr> {

    let (input, _) = bom(input)?; // Consume BOM if present
    let (input, result) = alt((
        map(|i| statementlist(i, string_manager), Expr::Statementlist),
        //map(|i| valuelist(i, string_manager), Expr::Valuelist),
    )).parse(input)?;
    Ok((input, result))
}

pub fn parse_raw(source: &str, string_manager: &StringResourceManager) -> (Expr, Vec<Error>) {
    let source = source.trim();
    let error_ctx = ErrorContext::new();
    let span = Span::new_extra(source, State(&error_ctx));

    let (remaining, stmts) = all(span, string_manager).expect("Parsing failed");

    // Report any trailing unparsed input
    if !remaining.fragment().trim().is_empty() {
        remaining.extra.0.add_error(Error(
            remaining.to_range(),
            "unexpected trailing input".to_string(),
        ));
    }

    (stmts, error_ctx.errors.into_inner())
}

pub fn parse(source: &str, string_manager: &StringResourceManager) -> (Expr, Vec<Error>) {
    let source = source.trim();

    let error_ctx = ErrorContext::new();
    let span = Span::new_extra(source, State(&error_ctx));

    // Run the parser
    match all(span, string_manager) {
        Ok((remaining, parsed_result)) => {
            // Check for unparsed input
            if !remaining.fragment().trim().is_empty() {
                remaining.extra.0.add_error(Error(
                    remaining.to_range(),
                    "unexpected trailing input".into(),
                ));
            }
            (parsed_result, error_ctx.errors.into_inner())
        }
        Err(nom::Err::Error(_)) | Err(nom::Err::Failure(_)) => {
            error_ctx.add_error(Error(
                span.to_range(),
                "Parsing failed - check syntax for this file".into()
            ));
            (Expr::default(), error_ctx.errors.into_inner())
        }
        Err(nom::Err::Incomplete(_)) => {
            error_ctx.add_error(Error(
                span.to_range(),
                "input incomplete and more data is needed".into()
            ));
            (Expr::default(), error_ctx.errors.into_inner())
        }
    }

}