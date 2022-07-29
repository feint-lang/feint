use num_bigint::BigInt;

use crate::format::FormatStrErr::*;
use crate::format::FormatStrToken::*;
use crate::format::*;
use crate::scanner::{Token, TokenWithLocation};
use crate::util::Location;

fn scan_ok(string: &str, expected_num_tokens: usize) -> Vec<FormatStrToken> {
    let result = scan_format_string(string);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    assert_eq!(tokens.len(), expected_num_tokens);
    tokens
}

#[test]
fn scan_simple() {
    let tokens = scan_ok("{1}", 1);
    let token = tokens.first().unwrap();
    let expected = Expr(vec![
        TokenWithLocation::new(
            Token::Int(BigInt::from(1)),
            Location::new(1, 1),
            Location::new(1, 1),
        ),
        TokenWithLocation::new(
            Token::EndOfStatement,
            Location::new(1, 2),
            Location::new(1, 2),
        ),
    ]);
    assert_eq!(token, &expected);
}

#[test]
fn scan_two_expr() {
    let tokens = scan_ok("a{1}b{'2'}c", 5);
    let mut token;

    token = tokens.get(0).unwrap();
    let expected = Str("a".to_owned());
    assert_eq!(token, &expected);

    token = tokens.get(1).unwrap();
    let expected = Expr(vec![
        TokenWithLocation::new(
            Token::Int(BigInt::from(1)),
            Location::new(1, 1),
            Location::new(1, 1),
        ),
        TokenWithLocation::new(
            Token::EndOfStatement,
            Location::new(1, 2),
            Location::new(1, 2),
        ),
    ]);
    assert_eq!(token, &expected);

    token = tokens.get(2).unwrap();
    let expected = Str("b".to_owned());
    assert_eq!(token, &expected);

    token = tokens.get(3).unwrap();
    let expected = Expr(vec![
        TokenWithLocation::new(
            Token::Str("2".to_owned()),
            Location::new(1, 1),
            Location::new(1, 3),
        ),
        TokenWithLocation::new(
            Token::EndOfStatement,
            Location::new(1, 4),
            Location::new(1, 4),
        ),
    ]);
    assert_eq!(token, &expected);

    token = tokens.get(4).unwrap();
    let expected = Str("c".to_owned());
    assert_eq!(token, &expected);
}

#[test]
fn scan_complex() {
    scan_ok("aaa{1 + 1}bbb{2 + 2}ccc{$'{3 + 3}xxx{4 + 4}'}ddd", 7);
}

#[test]
fn scan_with_tuple() {
    scan_ok("{(1, 2, 3, 'a', 'b', 'c')}", 1);
}

#[test]
fn scan_escaped_brackets() {
    let tokens = scan_ok("\\{\\}", 1);
    let token = tokens.last().unwrap();
    let expected = Str("{}".to_owned());
    assert_eq!(token, &expected);
}

#[test]
fn scan_no_expr() {
    let tokens = scan_ok("abc", 1);
    let token = tokens.last().unwrap();
    let expected = Str("abc".to_owned());
    assert_eq!(token, &expected);
}

#[test]
fn scan_empty_expr() {
    let result = scan_format_string("{}");
    assert_eq!(result, Err(EmptyExpr(0)));
}

#[test]
fn scan_unmatched_opening_bracket() {
    let result = scan_format_string("{1");
    assert_eq!(result, Err(UnmatchedOpeningBracket(2)));
    let result = scan_format_string("a{1");
    assert_eq!(result, Err(UnmatchedOpeningBracket(3)));
}

#[test]
fn scan_unmatched_closing_bracket() {
    let result = scan_format_string("1}");
    assert_eq!(result, Err(UnmatchedClosingBracket(3)));
    let result = scan_format_string("a1}");
    assert_eq!(result, Err(UnmatchedClosingBracket(4)));
}
