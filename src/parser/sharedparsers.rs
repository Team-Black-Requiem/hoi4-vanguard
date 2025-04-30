use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while1, take_while_m_n},
    character::complete::{char, char as nom_char, digit1, multispace0, multispace1, not_line_ending, satisfy},
    combinator::{map, not, opt, peek, recognize},
    error::{context, Error, ParseError},
    multi::{many0, many1},
    sequence::{delimited, preceded, terminated, tuple},
    Err,
    IResult
};
use nom_locate::LocatedSpan;
use log::info;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::utility::position::Range;
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

// Utility functions
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

/// Matches one or more characters that are NOT '\' or '"'
fn quoted_char_snippet(input: &str) -> IResult<&str, &str> {
    log::debug!("THIS IS QUOTED_CHAR_SNIPPET. Parsing quoted_char_snippet: {:?}", truncate_input(input, 2));
    take_while1(|c: char| c != '\\' && c != '"'

)(input)
}

/// Matches an escaped sequence: either `\"` or `\`
fn escaped_char(input: &str) -> IResult<&str, &str> {
    log::debug!("THIS IS ESCAPED_CHAR. Parsing escaped_char: {:?}", truncate_input(input, 2));
    map(
        alt((tag("\\\""), tag("\\"))),
        |s: &str| s,
    )(input)
}

fn metaprogramming_char_snippet(input: &str) -> IResult<&str, &str> {
    is_not("]\\")(input)
}

// A simple version of between_l that uses nom::error::Error.
pub fn old_between_l<'a, F, G, H, O1, O2, O3>(
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

// A simple version of between_l that uses nom::error::Error.
pub fn between_l<'a, F, G, H, O1, O2, O3>(
    mut popen: F,
    mut pclose: G,
    mut p: H,
    label: &'static str,
) -> impl FnMut(&'a str) -> IResult<&'a str, O2, Error<&'a str>>
where
    F: FnMut(&'a str) -> IResult<&'a str, O1, Error<&'a str>>,
    G: FnMut(&'a str) -> IResult<&'a str, O3, Error<&'a str>>,
    H: FnMut(&'a str) -> IResult<&'a str, O2, Error<&'a str>>,
{
    move |input: &'a str| {
        log::debug!("Parsing between_l ({}): {:?}", label, truncate_input(input, 2));

        // Match the opening delimiter
        let (input, _) = popen(input)?;

        // Parse the inner content
        let (remaining, output_inner) = p(input)?;

        // Attempt to match the closing delimiter
        match pclose(remaining) {
            Ok((remaining, _)) => {
                log::debug!("Successfully matched closing delimiter for {}", label);
                Ok((remaining, output_inner))
            }
            Err(_) => {
                // Check if the remaining input is EOF
                match nom::combinator::eof::<_, Error<&str>>(remaining) {
                    Ok((remaining, _)) => {
                        log::warn!(
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
fn clause<'a, O, F>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, Error<&'a str>>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, Error<&'a str>>,
{
    between_l(nom_char('{'), nom_char('}'), inner, "clause")
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


fn operator(input: &str) -> IResult<&str, Operator> {
    log::debug!("THIS IS OPERATOR. Parsing operator: {:?}", truncate_input(input, 2));
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
            |s: &str| s.trim().to_string(),                       // Convert the result to `String`
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
    log::debug!("THIS IS KEY. Parsing key: {:?}", truncate_input(input, 2));
    
    //log::debug!("THIS IS KEY. Parsed key: {:?}", res);
    parser(input)
}

// Key parser that matches a quoted key
fn key_q(input: &str) -> IResult<&str, Key> {
    log::debug!("Parsing key_q: {:?}", truncate_input(input, 2));
    map(
        quoted_string, // Use the `quoted_string` parser
        |s: String| Key::new(s), // Wrap the parsed string in a `Key` struct
    )(input)
}

fn value_s<'a>(
    input: &'a str,
    string_manager: &StringResourceManager,
) -> IResult<&'a str, Value> {
    log::debug!("THIS IS VALUE_S. Parsing value_s: {:?}", truncate_input(input, 2));

    // Check for a `"` at the beginning of the string and log a warning
    // we'll make this more robust later
    if peek(tag::<_, _, nom::error::Error<&str>>("\""))(input).is_ok() {
        log::warn!(
            "Detected a `\"` at the beginning of the string in value_s: {:?}",
            truncate_input(input, 2)
        );
    }
    context(
        "string",
        map(
            // Use `delimited` to discard the `"` at the beginning and end of the because if its here it escaped from value_q
            delimited(opt(tag("\"")),take_while1(is_value_char),opt(tag("\""))),
            |s: &str| Value::String(string_manager.intern_identifier_token(s)),
        ),
    )(input)
}

fn value_i(input: &str) -> IResult<&str, Value> {
    map(take_while_m_n(1, 10, |c: char| c.is_ascii_digit()), |s: &str| Value::Int(s.parse::<i32>().unwrap()))(input)
}

// A parser for floating point numbers (e.g., "123.456")
fn value_f(input: &str) -> IResult<&str, Value> {
    map(
        recognize(
            tuple((digit1, char('.'), digit1))
        ),
        |s: &str| Value::Float(s.parse::<f64>().unwrap())
    )(input)
}

fn value_b_yes(input: &str) -> IResult<&str, Value> {
    map(
        preceded(
            tag("yes"),
            peek(not(satisfy(|c| is_value_char(c) && c != '}'))), // Allow `yes` to be followed by `}` or whitespace
        ),
        |_| Value::Bool(true), // Return `Value::Bool(true)`
    )(input)
}

fn value_b_no(input: &str) -> IResult<&str, Value> {
    map(
        preceded(
            tag("no"),
            peek(not(satisfy(|c| is_value_char(c) && c != '}'))), // Allow `no` to be followed by `}` or whitespace
        ),
        |_| Value::Bool(false), // Return `Value::Bool(false)`
    )(input)
}

// Match a quoted string (with escape sequences) with additional checks
// this is a roundabout way to ensure that quotes are opened and closed properly
// this is a bit of a hack and likely still has edge cases
// it may consider a string to be valid even if it is not
// or vice versa
// better then the alternative of not enforcing string validity at all
fn quoted_string(input: &str) -> IResult<&str, String> {
    log::debug!("Parsing quoted_string: {:?}", truncate_input(input, 2));
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

        parser(input)
}

fn value_q<'a>(
    input: &'a str,
    string_manager: &StringResourceManager,
) -> IResult<&'a str, Value> {
    log::debug!("THIS IS VALUE_Q. Parsing value_q: {:?}", truncate_input(input, 2));
    map(
        quoted_string, // Parse the quoted string
        |s: String| {
            let token = string_manager.intern_identifier_token(&s); // Intern the string
            Value::QString(token) // Return it as a Value::QString
        },
    )(input)
}

fn hsv3(input: &str) -> IResult<&str, Value> {
    map(
        tuple((
            // First value: apply parse_with_position(value_f), then consume ws twice.
            terminated(terminated(parse_with_position(value_f), multispace0), multispace0),
            // Second value: apply parse_with_position(value_f), then consume ws.
            terminated(parse_with_position(value_f), multispace0),
            // Third value: apply parse_with_position(value_f), then consume ws.
            terminated(parse_with_position(value_f), multispace0),
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
            terminated(terminated(parse_with_position(value_f), multispace0), multispace0),
            // Second value: apply parse_with_position(value_f), then consume ws.
            terminated(parse_with_position(value_f), multispace0),
            // Third value: apply parse_with_position(value_f), then consume ws.
            terminated(parse_with_position(value_f), multispace0),
            // Fourth val
            terminated(parse_with_position(value_f), multispace0),
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
            terminated(terminated(parse_with_position(value_f), multispace0), multispace0),
            terminated(parse_with_position(value_f), multispace0),
            terminated(parse_with_position(value_f), multispace0),
            // Optional fourth value.
            opt(terminated(parse_with_position(value_f), multispace0)),
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
    terminated(hsv_i, multispace0)(input)
}

/// Parser for hsvC:
///   strSkip "HSV" >>. hsvI .>> ws
fn hsv_c(input: &str) -> IResult<&str, Value> {
    let (input, _) = str_skip("HSV")(input)?;
    terminated(hsv_i, multispace0)(input)
}

/// Parser for rgbI (a clause with 3 or 4 integer values).
fn rgb_i(input: &str) -> IResult<&str, Value> {
    map(
        tuple((
            terminated(terminated(parse_with_position(value_i), multispace0), multispace0),
            terminated(parse_with_position(value_i), multispace0),
            terminated(parse_with_position(value_i), multispace0),
            opt(terminated(parse_with_position(value_i), multispace0)),
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
            terminated(terminated(parse_with_position(value_i), multispace0), multispace0),
            terminated(parse_with_position(value_i), multispace0),
            terminated(parse_with_position(value_i), multispace0),
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
            terminated(terminated(parse_with_position(value_i), multispace0), multispace0),
            terminated(parse_with_position(value_i), multispace0),
            terminated(parse_with_position(value_i), multispace0),
            terminated(parse_with_position(value_i), multispace0),
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
    terminated(rgb_i, multispace0)(input)
}

/// Parser for rgbC:
///   strSkip "RGB" >>. rgbI .>> ws
fn rgb_c(input: &str) -> IResult<&str, Value> {
    let (input, _) = str_skip("RGB")(input)?;
    terminated(rgb_i, multispace0)(input)
}

fn metaprograming<'a>(
    input: &'a str,
    string_manager: &'a StringResourceManager,
) -> IResult<&'a str, Value> {
    map(
        tuple((tag("@\\["), metaprogramming_char_snippet, char(']'))),
        |(start, middle, end_char)| {
            // Concatenate the three pieces.
            let combined = format!("{}{}{}", start, middle, end_char);
            // Intern the concatenated string using the provided StringResourceManager.
            let token = string_manager.intern_identifier_token(&combined);
            // Wrap the interned token in the Value::String variant.
            Value::String(token)
        },
    )(input)
}

// A parser to obtain the current position using nom_locate.
fn get_position(input: &str) -> IResult<&str, LocatedSpan<&str>> {
    // Wrap the input in a LocatedSpan; since nom_locate works on the input,
    // we can simply return the current span.
    Ok((input, LocatedSpan::new(input)))
}

fn leaf_value<'a>(input: &'a str, string_manager: &'a StringResourceManager) -> IResult<&'a str, (Range, Value)> {
    log::debug!("Attempting to parse leaf_value from: {:?}", truncate_input(input, 2));
    
    // Capture starting position.
    let (input, start_span) = get_position(input)?;
    
    // Parse a value followed by trailing whitespace.
    let (input, val) = delimited(multispace0, |i| value(i, string_manager), multispace0)(input)?;
    
    // Lookahead: ensure the next token is NOT an operator.
    // If an operator is found, `not(peek(operator))` will fail without consuming input.
    let (input, _) = not(preceded(multispace0,peek(operator)))(input)?;
    
    // Capture ending position.
    let (input, end_span) = get_position(input)?;
    let range = get_range(start_span, end_span);
    
    log::debug!("Returning leaf_value: {:?}", (range, &val));
    Ok((input, (range, val)))
}

fn value_block<'a>(input: &'a str, string_manager: &'a StringResourceManager) -> IResult<&'a str, Value> {
    let inner = alt((
        // Map leaf_value into a Statement::Value.
        map(
            |input| leaf_value(input, string_manager),
            |(range, val)| Statement::Value(range, val),
        ),
        // Map comment into a Statement::Comment.
        map(comment, |s| Statement::Comment(s.0, s.1)),
    ));
    let (input, stmts) = many0(inner)(input)?;
    Ok((input, Value::Clause(stmts)))
}

fn value_clause<'a>(
    input: &'a str,
    string_manager: &'a StringResourceManager,
) -> IResult<&'a str, Value> {
    log::debug!("Parsing value_clause: {:?}", truncate_input(input, 5));

    let mut parser = preceded(
        peek(tag("{")),
        clause(delimited(
            multispace0,
            many0(|input| {
                log::debug!("Parsing nested statement in value_clause: {:?}", truncate_input(input, 2));
                statement(input, string_manager)
            }),
            multispace0,
        )),
    );

    parser(input).map(|(remaining, stmts)| (remaining, Value::Clause(stmts)))
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
fn value_custom<'a>(
    input: &'a str,
    string_manager: &'a StringResourceManager,
) -> IResult<&'a str, Value> {
    if input.trim().is_empty() {
        log::debug!("Input is empty, returning an error.");
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Eof)));
    }
    log::debug!("Parsing value: {:?}", truncate_input(input, 2));

    let mut parser = alt((
        // Use `peek` to check for specific starting characters or prefixes
        preceded(peek(tag("{")), |i| {
            log::debug!("Matched peek for value_clause");
            value_clause(i, string_manager)}),
        preceded(peek(tag("\"")), |i| value_q(i, string_manager)),
        preceded(
            peek(satisfy(|c| c.is_ascii_digit() || c == '-')),
            alt((value_f, value_i, |i| value_s(i, string_manager))),
        ),
        preceded(peek(tag("rgb")), rgb),
        preceded(peek(tag("RGB")), rgb_c),
        preceded(peek(tag("hsv")), hsv),
        preceded(peek(tag("HSV")), hsv_c),
        preceded(peek(tag("yes")), value_b_yes),
        preceded(peek(tag("no")), value_b_no),
        preceded(peek(tag("@\\")), |i| metaprograming(i, string_manager)),
        // Fallback to value_s checking for unbalanced quotes before parsing
        |i| value_s(i, string_manager),
    ));

    parser(input)
}

// ==================================================================
// keyvalue parser
// ==================================================================
//
// keyvalue = pipe5 getPosition (keyQ <|> key) operator value (getPosition .>> ws)
//             (fun start id op value endp ->
//                  KeyValue(PosKeyValue(getRange start endp, KeyValueItem(id, value, op))))
fn keyvalue_parser<'a>(
    input: &'a str,
    string_manager: &'a StringResourceManager,
) -> IResult<&'a str, Statement> {
    log::debug!("Parsing keyvalue: {:?}", truncate_input(input, 2));

    let (input, start_span) = get_position(input)?;

    // Use `peek` to ensure the input starts with a valid key
    let (input, id) = preceded(peek(alt((key_q, key))), alt((key_q, key)))(input)?;
    log::debug!("Parsed key: {:?}", id.to_string());

    let (input, op) = operator(input)?;
    log::debug!("Parsed operator: {:?}", op);

        // Allow an optional comment and newline before the value clause
        // edge case handling
        // might be a terrible idea to implement this way
        let (input, _) = opt(terminated(comment, multispace0))(input)?;

    let (input, val) = value(input, string_manager)?;
    log::debug!("Parsed value: {:?}", val.to_string(string_manager));

    let (input, end_span) = get_position(input)?;

    let range = get_range(start_span, end_span);
    let kv_item = KeyValueItem {
        key: id,
        value: val,
        operator: op,
    };
    log::debug!("Created KeyValueItem: {:?}", kv_item);

    Ok((input, Statement::KeyValue(PosKeyValue { range, kv_item })))
}

fn value<'a>(input: &'a str, string_manager: &'a StringResourceManager) -> IResult<&'a str, Value> {
    // We delegate to our custom value parser.
    value_custom(input, string_manager)
}

fn keyvalue<'a>(input: &'a str, manager: &'a StringResourceManager) -> IResult<&'a str, Statement> {
    keyvalue_parser(input, manager)
}

fn statement<'a>(
    input: &'a str,
    string_manager: &'a StringResourceManager,
) -> IResult<&'a str, Statement> {
    let input = input.trim(); // Trim leading and trailing whitespace

    if input.is_empty() {
        log::debug!("Input is empty, returning an error.");
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Eof)));
    }
    log::debug!("Parsing statement: {:?}", truncate_input(input, 2));

    let parse_comment = preceded(peek(tag("#")), map(comment, |s| {
        log::debug!("Parsed comment: {:?}", s);
        Statement::Comment(s.0, s.1)
    }));

    let parse_leaf_value = preceded(
        peek(not(peek(operator_lookahead))),
        map(|i| leaf_value(i, string_manager), |(r, v)| Statement::Value(r, v)),
    );

    let parse_keyvalue = preceded(peek(alt((key_q, key))), |i| keyvalue(i, string_manager));

    terminated(alt((parse_comment, parse_leaf_value, parse_keyvalue)), multispace0)(input)
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

fn alle<'a>(input: &'a str, string_manager: &'a StringResourceManager) -> IResult<&'a str, ParsedFile> {
    multispace0(input)?; // Consume leading whitespace
    let (input, stmts) = many0(|i| statement(i, string_manager))(input.trim())?;
    let (input, _) = nom::combinator::eof(input)?;
    Ok((input, ParsedFile { statements: stmts }))
}

/// Parses one or more items, each of which is either a comment or a leaf value.
/// Each comment is mapped to Statement::Comment and each leaf value is mapped to Statement::Value.
/// The parser succeeds only if all input is consumed (via `eof`).
fn valuelist<'a>(input: &'a str, string_manager: &'a StringResourceManager) -> IResult<&'a str, Vec<Statement>> {
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
    )(input);
    
    log::debug!("valuelist parsed: {:?}", result);
    result
}


fn statementlist<'a>(input: &'a str, string_manager: &'a StringResourceManager) -> IResult<&'a str, Vec<Statement>> {
    let input = input.trim(); // Trim leading and trailing whitespace

    if input.is_empty() {
        log::debug!("Input is empty, checking for EOF.");
        return nom::combinator::eof(input).map(|(remaining, _)| (remaining, vec![]));
    }

    let (input, stmts) = many0(|i| statement(i, string_manager))(input.trim())?;
    log::debug!(
        "Remaining input in statementlist (first 2 lines):\n{}",
        truncate_input(input, 5)
    );

    // Ensure all input is consumed
    let (input, _) = multispace0(input)?;
    let (input, _) = nom::combinator::eof(input)?;
    Ok((input, stmts))
}

// For the top–level parser “all”, we assume that if valuelist fails we try statementlist.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum AllResult {
    Valuelist(Vec<Statement>),
    Statementlist(Vec<Statement>),
}

impl AllResult {
    pub fn default() -> Self {
        AllResult::Statementlist(vec![])
    }
}

fn bom(input: &str) -> IResult<&str, ()> {
    opt(tag("\u{feff}"))(input).map(|(next_input, _)| (next_input, ()))
}

pub(crate) fn all<'a>(input: &'a str, string_manager: &'a StringResourceManager) -> IResult<&'a str, AllResult> {
    let input = input.trim(); // Trim leading and trailing whitespace

    if input.is_empty() {
        log::debug!("Input is empty, checking for EOF.");
        return nom::combinator::eof(input).map(|(remaining, _)| (remaining, AllResult::default()));
    }

    let (input, _) = bom(input)?; // Consume BOM if present
    let (input, _) = multispace0(input)?; // Consume leading whitespace
    let (input, result) = alt((
        map(|i| statementlist(i, string_manager), AllResult::Statementlist),
        //map(|i| valuelist(i, string_manager), AllResult::Valuelist),
    ))(input)?;
    let (input, _) = multispace0(input)?; // Consume trailing whitespace or newlines
    let (input, _) = nom::combinator::eof(input)?; // Ensure EOF
    Ok((input, result))
}

#[test]
fn test_single_line_clause() {
    let string_manager = crate::utility::util::StringResourceManager::new();
    let input = r#"{ has_dlc = "No Step Back" }"#;
    let result = value_clause(input, &string_manager);
    assert!(result.is_ok());
}

#[test]
fn test_multi_line_clause() {
    let string_manager = crate::utility::util::StringResourceManager::new();
    let input = r#"{
        original_tag = SOV
        has_dlc = "La Resistance"
    }"#;
    let result = value_clause(input, &string_manager);
    assert!(result.is_ok());
}

#[test]
fn test_nested_enable_block() {
    let string_manager = crate::utility::util::StringResourceManager::new();
    let input = r#"{

    	allowed = { has_dlc = "No Step Back" }
    	enable = {
    		SOV = { SOV_is_exiles = yes}
    		NOT = {
    			tag = SOV
    		}
    	}
    }
    "#;
    let result = value_clause(input, &string_manager);
    assert!(result.is_ok(), "Failed to parse nested enable block: {:?}", result);
}

#[test]
fn test_value_clause_simple() {
    let string_manager = crate::utility::util::StringResourceManager::new();
    let input = r#"{ key = value }"#;
    let result = value_clause(input, &string_manager);
    assert!(result.is_ok(), "Failed to parse simple clause: {:?}", result);
}

#[test]
fn test_value_clause_nested() {
    let string_manager = crate::utility::util::StringResourceManager::new();
    let input = r#"{ key = { nested_key = nested_value } }"#;
    let result = value_clause(input, &string_manager);
    assert!(result.is_ok(), "Failed to parse nested clause: {:?}", result);
}

#[test]
fn test_value_clause_complex() {
    let string_manager = crate::utility::util::StringResourceManager::new();
    let input = r#"{ key = { nested_key = { deep_key = deep_value } } }"#;
    let result = value_clause(input, &string_manager);
    assert!(result.is_ok(), "Failed to parse complex clause: {:?}", result);
}

#[test]
fn test_between_l_with_nested_input() {
    let input = r#"{

        allowed = { has_dlc = "No Step Back" }
        enable = {
            SOV = { SOV_is_exiles = yes}
            NOT = {
                tag = SOV
            }
        }
    }"#;

    // Define a recursive parser for the inner content
    fn recursive_inner_parser(input: &str) -> IResult<&str, String> {
        map(
            many0(alt((
                // Match nested braces recursively
                map(
                    between_l(
                        nom::character::complete::char('{'),
                        nom::character::complete::char('}'),
                        recursive_inner_parser,
                        "nested_braces",
                    ),
                    |nested| format!("{{{}}}", nested),
                ),
                // Match any other characters
                map(is_not("{}"), |s: &str| s.to_string()),
            ))),
            |parts| parts.concat(), // Combine all parts into a single string
        )(input)
    }

    // Use `between_l` to parse the content between `{` and `}`
    let mut parser = between_l(
        nom::character::complete::char('{'),
        nom::character::complete::char('}'),
        recursive_inner_parser,
        "test_between_l",
    );

    let result = parser(input);

    // Assert that the result is successful
    assert!(result.is_ok(), "Failed to parse input with between_l: {:?}", result);

    // Optionally, print the parsed result for debugging
    if let Ok((remaining, parsed)) = result {
        println!("Remaining input: {:?}", remaining);
        println!("Parsed content: {:?}", parsed);
    }
}

#[test]
fn test_escaped_char() {
    // Initialize the logger for the test
    let _ = flexi_logger::Logger::try_with_str("debug").unwrap()
    .log_to_file(flexi_logger::FileSpec::default().directory(std::path::PathBuf::from(".")))
    .duplicate_to_stderr(flexi_logger::Duplicate::Info)  
    .format_for_files(flexi_logger::colored_with_thread)
    .start();

    let string_manager = crate::utility::util::StringResourceManager::new();
    let input = r#"
        create_unit = {
            division = "division_template =\"Pashtun Levy\" start_experience_factor = 0.4 start_equipment_factor = 1.0"
            owner = AFG
            count = 1			
            prioritize_location = 10737
        }
    "#;
    let result = all(input, &string_manager);
    assert!(result.is_ok(), "Parsing failed: {:?}", result);
}

#[test]
fn test_unclosed_top_level_bracket() {
    let string_manager = crate::utility::util::StringResourceManager::new();
    let input = r#"
        BRA_fnm_organization = {
            include = generic_motorized_mechanized_organization
            icon = GFX_idea_BRA_fnm
            allowed = { 
                has_dlc = "Trial of Allegiance"
                tag = BRA
            }
            available = { 
                IF = {
                    limit = {
                        FROM = { NOT = { original_tag = BRA } }
                    }
                    FROM = { NOT = { has_war_with = BRA } }
                }
                ELSE = {
                    FROM = { 
                        OR = { 
                            has_completed_focus = SMB_motorized 
                            has_completed_focus = BRA_fabrica_nacional_de_motores
                        }
                    }
                }
            }
        "#;

    let result = all(input, &string_manager);
    println!("Result: {:?}", result);
    assert!(result.is_ok(), "Failed to parse unclosed top-level bracket: {:?}", result);
}

#[test]
fn test_unopened_quote() {
    let string_manager = crate::utility::util::StringResourceManager::new();
    let input = r#"Pohjois-Uudenmaan suojeluskuntapiiri" "#;
    let result = value_custom(input, &string_manager);
    assert!(result.is_err(), "Expected error for unopened quote, got: {:?}", result);
}

#[test]
fn test_unbalanced_quotes() {
    let string_manager = crate::utility::util::StringResourceManager::new();
    let input = r#""Pohjois-Uudenmaan suojeluskuntapiiri"#;
    let result = value_custom(input, &string_manager);
    assert!(result.is_err(), "Expected error for unbalanced quotes, got: {:?}", result);
}

#[test]
fn test_unbalanced_quotes_all() {
    let _ = flexi_logger::Logger::try_with_str("debug").unwrap()
    .log_to_file(flexi_logger::FileSpec::default().directory(std::path::PathBuf::from(".")))
    .duplicate_to_stderr(flexi_logger::Duplicate::Info)  
    .format_for_files(flexi_logger::colored_with_thread)
    .start();
    let string_manager = crate::utility::util::StringResourceManager::new();
    let input = r#"		
        7 = { "Nylands Södra skyddskårsdistrikt" } #Helsinki
		8 = { "Etelä-Kymenlaakson suojeluskuntapiiri" } #Kotka
		9 = { "Pohjois-Kymenlaakson suojeluskuntapiiri" } #Kouvola
		10 = { Pohjois-Uudenmaan suojeluskuntapiiri" } #Kerava"
		11 = { Suur-Saimaan suojeluskuntapiiri" } #Lappeenranta"
		12 = { Lahden suojeluskuntapiiri" } #Lahti "
		13 = { Lahden suojeluskuntapiiri" } #Lahti"
		14 = { Kanta-Hämeen suojeluskuntapiiri" } #Hämeenlinna"
		15 = { Lounais-Hämeen suojeluskuntapiiri" } #Forssa"
		16 = { "Pirkka-Hämeen suojeluskuntapiiri" } #Tampere"#;
    let result = value_custom(input, &string_manager);
    assert!(result.is_err(), "Expected error for unbalanced quotes, got: {:?}", result);
}