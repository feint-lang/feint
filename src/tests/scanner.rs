use crate::scanner;
use crate::scanner::{Location, Token, TokenWithLocation};

#[test]
fn scan_empty() {
    let tokens = scan("");
    assert_eq!(tokens.len(), 1);
    check_token(tokens.get(0), Token::EndOfInput, 1, 1, 1, 1);
}

#[test]
fn scan_int() {
    let tokens = scan("123");
    assert_eq!(tokens.len(), 3);
    check_token(tokens.get(0), Token::Int("123".to_string(), 10), 1, 1, 1, 3);
}

#[test]
fn scan_binary_number() {
    let tokens = scan("0b11");
    assert_eq!(tokens.len(), 3);
    check_token(tokens.get(0), Token::Int("11".to_string(), 2), 1, 1, 1, 4);
}

#[test]
fn scan_float() {
    let tokens = scan("123.1");
    assert_eq!(tokens.len(), 3);
    check_token(tokens.get(0), Token::Float("123.1".to_string()), 1, 1, 1, 5);
}

#[test]
fn scan_float_with_e_and_no_sign() {
    let tokens = scan("123.1e1");
    assert_eq!(tokens.len(), 3);
    let expected = Token::Float("123.1E+1".to_string());
    check_token(tokens.get(0), expected, 1, 1, 1, 7);
}

#[test]
fn scan_float_with_e_and_sign() {
    let tokens = scan("123.1e+1");
    eprintln!("{:?}", tokens);
    assert_eq!(tokens.len(), 3);
    let expected = Token::Float("123.1E+1".to_string());
    check_token(tokens.get(0), expected, 1, 1, 1, 8);
}

#[test]
fn scan_string_with_embedded_quote() {
    // "\"abc"
    let source = "\"\\\"abc\"";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 3);
    check_string_token(tokens.get(0), "\"abc", 1, 1, 1, 7);
}

#[test]
fn scan_string_with_newline() {
    // "abc
    // "
    let source = "\"abc\n\"";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 2);
    check_string_token(tokens.get(0), "abc\n", 1, 1, 1, 5);
}

#[test]
fn scan_string_with_many_newlines() {
    // " a
    // b
    //
    // c
    //
    //
    //   "
    let source = "\" a\nb\n\nc\n\n\n  \"";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 3);
    check_string_token(tokens.get(0), " a\nb\n\nc\n\n\n  ", 1, 1, 7, 3);
}

#[test]
fn scan_string_unclosed() {
    let source = "\"abc";
    match scanner::scan(source) {
        Err((error_token, tokens)) => match error_token.token {
            Token::UnterminatedString(string) => {
                assert_eq!(tokens.len(), 0);
                assert_eq!(string, source.to_string());
                assert_eq!(error_token.start, Location::new(1, 1));
                let new_source = source.to_string() + "\"";
                match scanner::scan(new_source.as_str()) {
                    Ok(tokens) => {
                        assert_eq!(tokens.len(), 3);
                        check_string_token(tokens.get(0), "abc", 1, 1, 1, 5);
                    }
                    _ => assert!(false),
                }
            }
            _ => assert!(false),
        },
        _ => assert!(false),
    };
}

#[test]
fn scan_indents() {
    let source = "\
f (x) ->
    x
    1


g (y) ->
    y
";
    let tokens = scan(source);

    // Used to keep rustfmt from wrapping
    let mut token;

    // f
    token = Token::Identifier("f".to_string());
    check_token(tokens.get(0), token, 1, 1, 1, 1);
    check_token(tokens.get(1), Token::LeftParen, 1, 3, 1, 3);
    token = Token::Identifier("x".to_string());
    check_token(tokens.get(2), token, 1, 4, 1, 4);
    check_token(tokens.get(3), Token::RightParen, 1, 5, 1, 5);
    check_token(tokens.get(4), Token::FuncStart, 1, 7, 1, 8);
    check_token(tokens.get(5), Token::Newline, 1, 9, 1, 9);
    check_token(tokens.get(6), Token::Indent(1), 2, 1, 2, 4);
    token = Token::Identifier("x".to_string());
    check_token(tokens.get(7), token, 2, 5, 2, 5);
    check_token(tokens.get(8), Token::Newline, 2, 6, 2, 6);
    check_token(tokens.get(9), Token::Int("1".to_string(), 10), 3, 5, 3, 5);
    check_token(tokens.get(10), Token::Newline, 3, 6, 3, 6);
    check_token(tokens.get(11), Token::Dedent, 4, 0, 4, 0);
    check_token(tokens.get(11), Token::Newline, 4, 1, 4, 1);
    check_token(tokens.get(11), Token::Newline, 5, 1, 5, 1);

    // g
    // token = Token::Identifier("g".to_string());
    // check_token(tokens.get(8), token, 5, 1, 1, 1);
    // check_token(tokens.get(9), Token::LeftParen, 5, 3, 1, 1);
    // token = Token::Identifier("y".to_string());
    // check_token(tokens.get(10), token, 5, 4, 1, 1);
    // check_token(tokens.get(11), Token::RightParen, 5, 5, 1, 1);
    // check_token(tokens.get(12), Token::FuncStart, 5, 7, 1, 1);
    // check_token(tokens.get(13), Token::Indent(4), 6, 1, 1, 1);
    // token = Token::Identifier("y".to_string());
    // check_token(tokens.get(14), token, 6, 5, 1, 1);
    // check_token(tokens.get(15), Token::Indent(0), 7, 1, 1, 1);
}

#[test]
#[should_panic]
fn scan_unknown() {
    let source = "{}";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 3);
    check_token(tokens.get(0), Token::Unknown('{'), 1, 1, 1, 1);
    check_token(tokens.get(1), Token::Unknown('}'), 1, 2, 1, 1);
    check_token(tokens.get(2), Token::Indent(0), 1, 3, 1, 1);
}

// Utilities -----------------------------------------------------------

/// Scan source and return tokens.
fn scan(source: &str) -> Vec<TokenWithLocation> {
    match scanner::scan(source) {
        Ok(tokens) => tokens,
        Err((error_token, tokens)) => panic!("Scan failed unexpectedly: {}", error_token),
    }
}

/// Check token returned by scanner against expected token.
fn check_token(
    actual: Option<&TokenWithLocation>,
    expected: Token,
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
) {
    let start = Location::new(start_line, start_col);
    let end = Location::new(end_line, end_col);
    assert_eq!(actual, Some(&TokenWithLocation::new(expected, start, end)));
}

fn check_string_token(
    actual: Option<&TokenWithLocation>,
    expected_string: &str,
    expected_start_line: usize,
    expected_start_col: usize,
    expected_end_line: usize,
    expected_end_col: usize,
) {
    assert!(actual.is_some());
    match actual {
        Some(TokenWithLocation {
            token: Token::String(actual_string),
            start:
                Location {
                    line: actual_start_line,
                    col: actual_start_col,
                },
            end:
                Location {
                    line: actual_end_line,
                    col: actual_end_col,
                },
        }) => {
            assert_eq!(actual_string, expected_string);
            assert_eq!(actual_start_line, &expected_start_line);
            assert_eq!(actual_start_col, &expected_start_col);
            assert_eq!(actual_end_line, &expected_end_line);
            assert_eq!(actual_end_col, &expected_end_col);
        }
        _ => assert!(false),
    }
}
