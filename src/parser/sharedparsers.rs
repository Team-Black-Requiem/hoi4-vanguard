use nom::{
    branch::alt,
    bytes::complete::{escaped, is_a, is_not, tag, take_till1, take_while, take_while1, take_while_m_n},
    character::complete::{char, char as nom_char, digit1, multispace0, multispace1, none_of, not_line_ending, satisfy, space1},
    combinator::{map, not, opt, peek, recognize},
    error::{context, Error, ErrorKind, ParseError},
    multi::{many0, many1},
    sequence::{delimited, preceded, terminated, tuple},
    Err,
    IResult, InputIter
};
use nom_locate::LocatedSpan;
use log::info;
use std::fmt::Debug;

use crate::utility::position::{Pos, Range};
use crate::utility::util;
use crate::parser::types;

use self::util::*;
use self::types::*;


/// A wrapper function to add logging to a parser.
pub fn with_logging<'a, F, O, E>(
    mut parser: F,
    label: &'static str,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
    E: ParseError<&'a str>,
{
    move |input: &'a str| {
        info!("Entering parser: {}", label);
        let result = parser(input);
        match &result {
            Ok(_) => info!("Leaving parser: {} (Success)", label),
            Err(_) => info!("Leaving parser: {} (Error)", label),
        }
        result
    }
}

// Example usage:
//
// let parser_with_logging = with_logging(some_parser, "example_parser");
// let result = parser_with_logging("some input");



/// Custom error type with detailed context
#[derive(Debug)]
pub struct CustomParseError<'a> {
    input: &'a str,
    message: String,
}

impl<'a> ParseError<&'a str> for CustomParseError<'a> {
    fn from_error_kind(input: &'a str, _: nom::error::ErrorKind) -> Self {
        Self {
            input,
            message: format!("Error at input: {:?}", input),
        }
    }

    fn append(input: &'a str, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.message = format!("{}, ErrorKind: {:?}", other.message, kind);
        other
    }
}

/// Implement ParseError for `LocatedSpan<&str>`
impl<'a> ParseError<LocatedSpan<&'a str>> for CustomParseError<'a> {
    fn from_error_kind(input: LocatedSpan<&'a str>, kind: ErrorKind) -> Self {
        Self {
            input: input.fragment(),
            message: format!(
                "Error at line {}, column {}: {:?}",
                input.location_line(),
                input.get_column(),
                kind
            ),
        }
    }

    fn append(input: LocatedSpan<&'a str>, kind: ErrorKind, mut other: Self) -> Self {
        other.message = format!(
            "{}, {:?} at line {}, column {}",
            other.message,
            kind,
            input.location_line(),
            input.get_column()
        );
        other
    }
}

/// Adds a descriptive context to the error
pub fn with_context<'a, F, O>(
    mut parser: F,
    label: &'static str,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, CustomParseError<'a>>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, CustomParseError<'a>>,
{
    move |input: &'a str| {
        match parser(input) {
            Ok(result) => Ok(result),
            Err(nom::Err::Error(e)) => Err(nom::Err::Error(CustomParseError {
                input,
                message: format!("{}: {}", label, e.message),
            })),
            Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(CustomParseError {
                input,
                message: format!("{}: {}", label, e.message),
            })),
            Err(nom::Err::Incomplete(needed)) => Err(nom::Err::Incomplete(needed)),
        }
    }
}

// A simple version of between_l that uses nom::error::Error.
pub fn between_l<'a, F, G, H, O1, O2, O3>(
    mut popen: F,
    mut pclose: G,
    mut p: H,
    _label: &'static str, // label unused in this version
) -> impl FnMut(&'a str) -> IResult<&'a str, O2, Error<&'a str>>
where
    F: FnMut(&'a str) -> IResult<&'a str, O1, Error<&'a str>>,
    G: FnMut(&'a str) -> IResult<&'a str, O3, Error<&'a str>>,
    H: FnMut(&'a str) -> IResult<&'a str, O2, Error<&'a str>>,
{
    move |input: &'a str| {
        let (input, _) = popen(input)?;
        let (input, output_inner) = p(input)?;
        let (input, _) = pclose(input)?;
        Ok((input, output_inner))
    }
}

// Sets of chars
// =======
const WHITESPACE_TEXT_CHARS: &str = " \t\r\n";

const NORSE_CHARS: &[char] = &['ö', 'ð', 'æ', 'ó', 'ä', 'Þ', 'Å', 'Ö'];

const ID_CHAR_ARRAY: &[char] = &[
    '_', ':', '@', '.', '\"', '-', '\'', '[', ']', '!', '<', '>', '$', '^', '&', '|', util::MAGIC_CHAR,
];

const VALUE_CHAR_ARRAY: &[char] = &[
    '_', '.', '-', ':', ';', '\'', '[', ']', '@', '\'', '+', '`', '%', '/', '!', ',', '<', '>',
    '?', '$', 'š', 'Š', '’', '|', '^', '*', '&', util::MAGIC_CHAR,
];

// Utility functions
fn is_any_of_id_char(c: char) -> bool {
    ID_CHAR_ARRAY.contains(&c)
}

fn is_any_value_char(c: char) -> bool {
    VALUE_CHAR_ARRAY.contains(&c)
}

fn is_id_char(c: char) -> bool {
    c.is_alphabetic() || c.is_digit(10) || is_any_of_id_char(c)
}

fn is_value_char(c: char) -> bool {
    c.is_alphabetic() || c.is_digit(10) || is_any_value_char(c)
}

// Whitespace parser
fn ws(input: &str) -> IResult<&str, &str> {
    multispace0(input)
}

// Match a specific string, followed by optional whitespace
fn str_parser<'a>(s: &'static str) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
    move |input: &'a str| terminated(tag(s), multispace0)(input)
}

// Skip a specific string, followed by optional whitespace
fn str_skip<'a>(s: &'static str) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
    move |input: &'a str| terminated(tag(s), multispace0)(input).map(|(next_input, _)| (next_input, ()))
}

// Match a specific character, followed by optional whitespace
fn ch_parser(c: char) -> impl FnMut(&str) -> IResult<&str, char> {
    move |input: &str| terminated(nom_char(c), multispace0)(input)
}

// Skip a specific character, followed by optional whitespace
fn ch_skip(c: char) -> impl FnMut(&str) -> IResult<&str, ()> {
    move |input: &str| terminated(nom_char(c), multispace0)(input).map(|(next_input, _)| (next_input, ()))
}

/// A clause parser that expects the inner content to be between `{` and `}`.
fn clause<'a, O, F>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, Error<&'a str>>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, Error<&'a str>>,
{
    between_l(nom_char('{'), nom_char('}'), inner, "clause")
}

/// Matches one or more characters that are NOT '\' or '"'
fn quoted_char_snippet(input: &str) -> IResult<&str, &str> {
    log::debug!("THIS IS QUOTED_CHAR_SNIPPET. Parsing quoted_char_snippet: {:?}", input);
    take_while(|c: char| c != '\\' && c != '"')(input)
}

/// Matches an escaped sequence: either `\"` or `\`
fn escaped_char(input: &str) -> IResult<&str, &str> {
    log::debug!("THIS IS ESCAPED_CHAR. Parsing escaped_char: {:?}", input);
    map(
        alt((tag("\\\""), tag("\\"))),
        |s: &str| s,
    )(input)
}

fn metaprogramming_char_snippet(input: &str) -> IResult<&str, &str> {
    is_not("]\\")(input)
}

// Use `LocatedSpan` for input type to track positions
type Span<'a> = LocatedSpan<&'a str>;

/// Get the range from start and end spans
pub fn get_range<'a>(start: Span<'a>, end: Span<'a>) -> Range {
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

    Range::new(
        start_line,
        start_column,
        end_line,
        end_column,
    )
}

fn parse_with_position<'a, O, F>(mut parser: F) -> impl FnMut(&'a str) -> IResult<&'a str, (Range, O)>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    move |input: &'a str| {
        // Wrap the current input in a LocatedSpan.
        let input_loc = LocatedSpan::new(input);
        // Run the wrapped parser.
        let (remaining, result) = parser(input)?;
        // Wrap the remaining input in a LocatedSpan.
        let remaining_loc = LocatedSpan::new(remaining);
        // Compute the range from the start (input_loc) to the start of remaining_loc.
        let range = get_range(input_loc, remaining_loc);
        Ok((remaining, (range, result)))
    }
}

fn operator(input: &str) -> IResult<&str, Operator> {
    log::debug!("THIS IS OPERATOR. Parsing operator: {:?}", input);
    alt((
        map(delimited(multispace0, tag("<="), multispace0), |_| Operator::LessThanOrEqual),
        map(delimited(multispace0,tag(">="), multispace0), |_| Operator::GreaterThanOrEqual),
        map(delimited(multispace0,tag("!="), multispace0), |_| Operator::NotEqual),
        map(delimited(multispace0, tag("=="), multispace0), |_| Operator::EqualEqual),
        map(delimited(multispace0,tag("?="),multispace0), |_| Operator::QuestionEqual),
        map(delimited(multispace0, tag("<"), multispace0), |_| Operator::LessThan),
        map(delimited(multispace0, tag(">"), multispace0), |_| Operator::GreaterThan),
        map(delimited(multispace0, tag("="),multispace0), |_| Operator::Equals),
    ))(input)
}

fn operator_lookahead(input: &str) -> IResult<&str, &str> {
    // Use `peek` so that input is not consumed.
    nom::combinator::peek(alt((
        delimited(multispace0, tag("="),multispace0),
        delimited(multispace0, tag(">"),multispace0),
        delimited(multispace0, tag("<"),multispace0),
        delimited(multispace0, tag("!"),multispace0),
        delimited(multispace0, tag("?="),multispace0),
    )))(input)
}

/// Parses a comment and captures its positional metadata.
pub fn comment(input: &str) -> IResult<&str, (Range, String)> {
    parse_with_position(
        map(
            terminated(
                preceded(nom_char('#'), not_line_ending), // Match `#` and capture the rest of the line
                multispace0,                              // Consume trailing whitespace
            ),
            |s: &str| s.to_string(),                      // Convert the result to `String`
        )
    )(input)
}

/// Key parser that returns a `Key` struct
fn key(input: &str) -> IResult<&str, Key> {
    let key_parser = map(
        // Use take_while1 to require at least one character that satisfies is_id_char
        terminated(take_while1(is_id_char), multispace0),
        |s: &str| Key::new(s.to_string()),
    );
    let mut parser = preceded(multispace0, key_parser); // Skip leading whitespace
    log::debug!("THIS IS KEY. Parsing key: {:?}", input);
    let res = parser(input);
    log::debug!("THIS IS KEY. Parsed key: {:?}", res);
    res
}

// Key parser that matches a quoted key
fn key_q(input: &str) -> IResult<&str, Key> {
    let key_parser = terminated(
        delimited(
            char('"'), // Opening quote
            alt((quoted_char_snippet, escaped_char)), // Match either quoted or escaped characters
            char('"'), // Closing quote
    ),
    multispace0); // Consume trailing whitespace

    map(key_parser, |s: &str| {
        let combined: String = s.to_string(); // Combine the pieces into a single string
        Key::new(combined) // Wrap it in the `Key` struct
    })(input)
}

fn value_s(input: &str) -> IResult<&str, Value> {
    log::debug!("THIS IS VALUE_S. Parsing value_s: {:?}", input);
    context("string", map(
        take_while1(is_value_char),
        |s: &str| Value::String(STRING_RESOURCE_MANAGER.intern_identifier_token(s))
    ))(input)
}

fn value_i(input: &str) -> IResult<&str, Value> {
    map(take_while_m_n(1, 10, |c: char| c.is_digit(10)), |s: &str| Value::Int(s.parse::<i32>().unwrap()))(input)
}

// A parser for floating point numbers (e.g., "123.456")
fn value_f(input: &str) -> IResult<&str, Value> {
    map(preceded(digit1,preceded(char('.'),digit1)),
        |s: &str| Value::Float(s.parse::<f64>().unwrap())  // Convert the string to `f64` and wrap in `Value::Float`
        )(input)
}

fn value_b_yes(input: &str) -> IResult<&str, Value> {
    map(preceded(tag("yes"),satisfy(|c| !is_value_char(c)),),
        |_| Value::Bool(true)  // Return `Value::Bool(true)`
    )(input)
}

fn value_b_no(input: &str) -> IResult<&str, Value> {
    map(preceded(tag("no"),satisfy(|c| !is_value_char(c)),),
        |_| Value::Bool(false)  // Return `Value::Bool(false)`
    )(input)
}

// Match a quoted string (with escape sequences)
fn quoted_string(input: &str) -> IResult<&str, String> {
    let parser = delimited(multispace0,
        delimited(
            char('"'),  // Opening quote
            alt((quoted_char_snippet, escaped_char)), // Match quoted or escaped characters
            char('"'),  // Closing quote
        ),
    multispace0);  // Consume trailing whitespace
    map(parser, |s: &str| s.to_string())(input)  // Combine all matched parts into a single string
}

fn value_q<'a>(input: &'a str) -> IResult<&'a str, Value> {
    log::debug!("THIS IS VALUE_Q. Parsing value_q: {:?}", input);
    map(
        quoted_string,  // Parse the quoted string
        move |s: String| {
            let token = STRING_RESOURCE_MANAGER.intern_identifier_token(&s);  // Intern the string
            Value::QString(token)  // Return it as a Value::QString
        }
    )(input)
}

fn hsv3(input: &str) -> IResult<&str, Value> {
    map(
        tuple((
            // First value: apply parse_with_position(value_f), then consume ws twice.
            terminated(terminated(parse_with_position(value_f), ws), ws),
            // Second value: apply parse_with_position(value_f), then consume ws.
            terminated(parse_with_position(value_f), ws),
            // Third value: apply parse_with_position(value_f), then consume ws.
            terminated(parse_with_position(value_f), ws),
        )),
        |(a, b, c)| {
            Value::Clause(vec![
                Statement::Value(a.0, a.1),
                Statement::Value(b.0, b.1),
                Statement::Value(c.0, c.1),
            ])
        }
    )(input)
}

fn hsv4(input: &str) -> IResult<&str, Value> {
    map(
        tuple((
            // First value: apply parse_with_position(value_f), then consume ws twice.
            terminated(terminated(parse_with_position(value_f), ws), ws),
            // Second value: apply parse_with_position(value_f), then consume ws.
            terminated(parse_with_position(value_f), ws),
            // Third value: apply parse_with_position(value_f), then consume ws.
            terminated(parse_with_position(value_f), ws),
            // Fourth val
            terminated(parse_with_position(value_f), ws),
        )),
        |(a, b, c, d)| {
            Value::Clause(vec![
                Statement::Value(a.0, a.1),
                Statement::Value(b.0, b.1),
                Statement::Value(c.0, c.1),
                Statement::Value(d.0, d.1)
            ])
        }
    )(input)
}

/// Parser for hsvI (a clause with 3 or 4 float values).
fn hsv_i(input: &str) -> IResult<&str, Value> {
    map(
        tuple((
            // Each value: run parse_with_position(value_f) then consume whitespace.
            terminated(terminated(parse_with_position(value_f), ws), ws),
            terminated(parse_with_position(value_f), ws),
            terminated(parse_with_position(value_f), ws),
            // Optional fourth value.
            opt(terminated(parse_with_position(value_f), ws)),
        )),
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
    )(input)
}

/// Parser for hsv:
///   strSkip "hsv" >>. opt (strSkip "360") >>. hsvI .>> ws
fn hsv(input: &str) -> IResult<&str, Value> {
    let (input, _) = str_skip("hsv")(input)?;
    let (input, _) = opt(str_skip("360"))(input)?;
    terminated(hsv_i, ws)(input)
}

/// Parser for hsvC:
///   strSkip "HSV" >>. hsvI .>> ws
fn hsv_c(input: &str) -> IResult<&str, Value> {
    let (input, _) = str_skip("HSV")(input)?;
    terminated(hsv_i, ws)(input)
}

/// Parser for rgbI (a clause with 3 or 4 integer values).
fn rgb_i(input: &str) -> IResult<&str, Value> {
    map(
        tuple((
            terminated(terminated(parse_with_position(value_i), ws), ws),
            terminated(parse_with_position(value_i), ws),
            terminated(parse_with_position(value_i), ws),
            opt(terminated(parse_with_position(value_i), ws)),
        )),
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
    )(input)
}

/// Parser for rgb3 (a clause with exactly 3 integer values).
fn rgb3(input: &str) -> IResult<&str, Value> {
    map(
        tuple((
            terminated(terminated(parse_with_position(value_i), ws), ws),
            terminated(parse_with_position(value_i), ws),
            terminated(parse_with_position(value_i), ws),
        )),
        |(a, b, c)| {
            Value::Clause(vec![
                Statement::Value(a.0, a.1),
                Statement::Value(b.0, b.1),
                Statement::Value(c.0, c.1),
            ])
        }
    )(input)
}

/// Parser for rgb4 (a clause with exactly 4 integer values).
fn rgb4(input: &str) -> IResult<&str, Value> {
    map(
        tuple((
            terminated(terminated(parse_with_position(value_i), ws), ws),
            terminated(parse_with_position(value_i), ws),
            terminated(parse_with_position(value_i), ws),
            terminated(parse_with_position(value_i), ws),
        )),
        |(a, b, c, d)| {
            Value::Clause(vec![
                Statement::Value(a.0, a.1),
                Statement::Value(b.0, b.1),
                Statement::Value(c.0, c.1),
                Statement::Value(d.0, d.1),
            ])
        }
    )(input)
}

/// Parser for rgb:
///   strSkip "rgb" >>. rgbI .>> ws
fn rgb(input: &str) -> IResult<&str, Value> {
    let (input, _) = str_skip("rgb")(input)?;
    terminated(rgb_i, ws)(input)
}

/// Parser for rgbC:
///   strSkip "RGB" >>. rgbI .>> ws
fn rgb_c(input: &str) -> IResult<&str, Value> {
    let (input, _) = str_skip("RGB")(input)?;
    terminated(rgb_i, ws)(input)
}

fn metaprograming(input: &str) -> IResult<&str, Value> {
    map(
        tuple((tag("@\\["), metaprogramming_char_snippet, char(']'))),
        |(start, middle, end_char)| {
            // Concatenate the three pieces.
            let combined = format!("{}{}{}", start, middle, end_char);
            // Intern the concatenated string.
            let token = STRING_RESOURCE_MANAGER.intern_identifier_token(&combined);
            // Wrap the interned token in the Value::String variant.
            Value::String(token)
        }
    )(input)
}

// A parser to obtain the current position using nom_locate.
fn get_position(input: &str) -> IResult<&str, LocatedSpan<&str>> {
    // Wrap the input in a LocatedSpan; since nom_locate works on the input,
    // we can simply return the current span.
    Ok((input, LocatedSpan::new(input)))
}

/// A combinator that “attempts” a parser. If the parser fails (even after consuming input),
/// it converts the failure into a recoverable error so that alt() may try another branch.
fn attempt<'a, F, O, E>(mut parser: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
    E: ParseError<&'a str>,
{
    move |input: &'a str| {
        // Save the original input.
        let original = input;
        match parser(input) {
            Ok(res) => Ok(res),
            // Convert a Failure (committed error) into an Error (recoverable)
            Err(Err::Failure(e)) => Err(Err::Error(e)),
            Err(e) => Err(e),
        }
    }
}


fn leaf_value(input: &str) -> IResult<&str, (Range, Value)> {
    log::debug!("Attempting to parse leaf_value from: {:?}", input);
    
    // Capture starting position.
    let (input, start_span) = get_position(input)?;
    
    // Parse a value followed by trailing whitespace.
    let (input, val) = delimited(multispace0,value, multispace0)(input)?;
    
    // Lookahead: ensure the next token is NOT an operator.
    // If an operator is found, `not(peek(operator))` will fail without consuming input.
    let (input, _) = not(preceded(multispace0,peek(operator)))(input)?;
    
    // Capture ending position.
    let (input, end_span) = get_position(input)?;
    let range = get_range(start_span, end_span);
    
    log::debug!("Returning leaf_value: {:?}", (range, &val));
    Ok((input, (range, val)))
}

fn value_block(input: &str) -> IResult<&str, Value> {
    let inner = alt((
        // Map leaf_value into a Statement::Value.
        map(leaf_value, |(range, val)| Statement::Value(range, val)),
        // Map comment into a Statement::Comment.
        map(comment, |s| Statement::Comment(s.0, s.1)),
    ));
    let (input, stmts) = many1(inner)(input)?;
    Ok((input, Value::Clause(stmts)))
}

fn value_clause(input: &str) -> IResult<&str, Value, Error<&str>> {
    map(clause(many1(statement)), |stmts| Value::Clause(stmts))(input)
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
fn value_custom(input: &str) -> IResult<&str, Value> {
    log::debug!("THIS IS VALUE_CUSTOM Parsing value: {:?}", input);
    if let Some(first) = input.chars().next() {
        match first {
            '{' => {
                log::debug!("Dispatching to value_clause");
                value_clause(input)
            },
            '"' => {
                log::debug!("Dispatching to value_q");
                value_q(input)
            },
            x if x.is_ascii_digit() || x == '-' => {
                log::debug!("Attempting to parse as integer or float");
                // Try integer, then float, then fallback.
                if let Ok((rest, v)) = value_i(input) {
                    log::debug!("Parsed as integer: {:?}", v);
                    Ok((rest, v))
                } else if let Ok((rest, v)) = value_f(input) {
                    log::debug!("Parsed as float: {:?}", v);
                    Ok((rest, v))
                } else {
                    log::debug!("Falling back to value_s");
                    value_s(input)
                }
            },
            _ => {
                log::debug!("Checking for specific prefixes");
                // Look at prefixes:
                if input.starts_with("rgb") {
                    log::debug!("Dispatching to rgb");
                    rgb(input)
                } else if input.starts_with("RGB") {
                    log::debug!("Dispatching to rgb_c");
                    rgb_c(input)
                } else if input.starts_with("hsv") {
                    log::debug!("Dispatching to hsv");
                    hsv(input)
                } else if input.starts_with("HSV") {
                    log::debug!("Dispatching to hsv_c");
                    hsv_c(input)
                } else if input.starts_with("yes") {
                    log::debug!("Dispatching to value_b_yes or value_s");
                    value_b_yes(input).or_else(|_| value_s(input))
                } else if input.starts_with("no") {
                    log::debug!("Dispatching to value_b_no or value_s");
                    value_b_no(input).or_else(|_| value_s(input))
                } else if input.starts_with("@\\[") {
                    log::debug!("Dispatching to metaprograming");
                    metaprograming(input)
                } else {
                    log::debug!("Falling back to value_s");
                    value_s(input)
                }
            }
        }
    } else {
        Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Eof)))
    }
}

// ==================================================================
// keyvalue parser
// ==================================================================
//
// keyvalue = pipe5 getPosition (keyQ <|> key) operator value (getPosition .>> ws)
//             (fun start id op value endp ->
//                  KeyValue(PosKeyValue(getRange start endp, KeyValueItem(id, value, op))))
fn keyvalue_parser(input: &str) -> IResult<&str, Statement> {
    log::debug!("THIS IS KEYVALUE_PARSER Parsing keyvalue: {:?}", input);
    let (input, start_span) = get_position(input)?;
    log::debug!("Parsed start position: {:?}", start_span);
    
    // Parse key: try key_q first, then key.
    let (input, id) = alt((key_q, key))(input)?;
    log::debug!("Parsed key: {:?}", id);

    let (input, op) = operator(input)?;
    log::debug!("Parsed operator: {:?}", op);

    let (input, val) = value(input)?;
    log::debug!("Parsed value: {:?}", val);
    
    let (input, end_span) = terminated(get_position, ws)(input)?;
    log::debug!("Parsed end position: {:?}", end_span);
    
    let range = get_range(start_span, end_span);
    let kv_item = KeyValueItem { key: id, value: val, operator: op };
    log::debug!("Created KeyValueItem: {:?}", kv_item);
    
    Ok((input, Statement::KeyValue(PosKeyValue { range, kv_item })))
}

fn value(input: &str) -> IResult<&str, Value> {
    // We delegate to our custom value parser.
    value_custom(input)
}

fn keyvalue(input: &str) -> IResult<&str, Statement> {
    keyvalue_parser(input)
}

fn statement(input: &str) -> IResult<&str, Statement> {
    log::debug!("THIS IS STATEMENT Parsing statement: {:?}", input);
    let result = alt((
        map(preceded(multispace0, comment), |s| {
            log::debug!("Parsed comment: {:?}", s);
            Statement::Comment(s.0, s.1)
        }),
        map(
            delimited(multispace0, leaf_value, not(operator_lookahead)),
            |(range, val)| {
                log::debug!("Parsed leaf_value: {:?}", val);
                Statement::Value(range, val)
            }
        ),
        keyvalue,
    ))(input);
    
    match &result {
        Ok((remaining_input, statement)) => {
            log::debug!("Successfully parsed statement: {:?}", statement);
            log::debug!("Remaining input: {:?}", remaining_input);
        },
        Err(err) => {
            //log::debug!("Failed to parse statement: {:?}", err);
        }
    }
    
    result
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
// We define ParsedFile and AllResult accordingly.
#[derive(Debug)]
pub struct ParsedFile {
    statements: Vec<Statement>,
}

fn alle(input: &str) -> IResult<&str, ParsedFile> {
    let (input, _) = ws(input)?;
    let (input, stmts) = many0(statement)(input)?;
    let (input, _) = nom::combinator::eof(input)?;
    Ok((input, ParsedFile { statements: stmts }))
}

/// Parses one or more items, each of which is either a comment or a leaf value.
/// Each comment is mapped to Statement::Comment and each leaf value is mapped to Statement::Value.
/// The parser succeeds only if all input is consumed (via `eof`).
fn valuelist(input: &str) -> IResult<&str, Vec<Statement>> {
    let result = many1(
        terminated(
            alt((
                map(comment, |(range, text)| Statement::Comment(range, text)),  
                map(leaf_value, |(range, value)| Statement::Value(range, value)), 
            )),
            multispace0,
        )
    )(input);
    
    log::debug!("valuelist parsed: {:?}", result);
    result
}


fn statementlist(input: &str) -> IResult<&str, Vec<Statement>> {
    let (input, stmts) = many0(statement)(input)?;
    let (input, _) = nom::combinator::eof(input)?;
    Ok((input, stmts))
}

// For the top–level parser “all”, we assume that if valuelist fails we try statementlist.
#[derive(Debug)]
pub(crate) enum AllResult {
    Valuelist(Vec<Statement>),
    Statementlist(Vec<Statement>),
}

pub(crate) fn all(input: &str) -> IResult<&str, AllResult> {
    let (input, _) = ws(input)?;
    alt((
        // Wrap the entire valuelist branch with attempt().
        map(valuelist, AllResult::Valuelist),
        map(statementlist, AllResult::Statementlist),
    ))(input)
}