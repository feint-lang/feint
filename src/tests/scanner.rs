use crate::scanner;
use crate::scanner::Scanner;
use crate::tokens::{Token, TokenWithPosition};

#[test]
fn scan_empty() {
    let tokens = scan("");
    assert_eq!(tokens.len(), 0);
}

#[test]
fn scan_int() {
    let tokens = scan("123");
    assert_eq!(tokens.len(), 2);
    check_token(tokens.get(0), Token::Int("123".to_string()), 1, 1);
}

#[test]
fn scan_float() {
    let tokens = scan("123.1");
    assert_eq!(tokens.len(), 2);
    check_token(tokens.get(0), Token::Float("123.1".to_string()), 1, 1);
}

#[test]
fn scan_float_with_e_and_no_sign() {
    let tokens = scan("123.1e1");
    eprintln!("{:?}", tokens);
    assert_eq!(tokens.len(), 2);
    check_token(tokens.get(0), Token::Float("123.1E+1".to_string()), 1, 1);
}

#[test]
fn scan_float_with_e_and_sign() {
    let tokens = scan("123.1e+1");
    eprintln!("{:?}", tokens);
    assert_eq!(tokens.len(), 2);
    check_token(tokens.get(0), Token::Float("123.1E+1".to_string()), 1, 1);
}

#[test]
fn scan_string_with_embedded_quote() {
    // "\"abc"
    let source = "\"\\\"abc\"";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 2);
    check_string_token(tokens.get(0), "\"abc", 1, 1, 4);
    check_token(tokens.get(1), Token::Indent(0), 1, 8);
}

#[test]
fn scan_string_with_newline() {
    // "abc
    // "
    let source = "\"abc\n\"";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 2);
    check_string_token(tokens.get(0), "abc\n", 1, 1, 4);
    check_token(tokens.get(1), Token::Indent(0), 2, 2);
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
    assert_eq!(tokens.len(), 2);
    check_string_token(tokens.get(0), " a\nb\n\nc\n\n\n  ", 1, 1, 12);
    check_token(tokens.get(1), Token::Indent(0), 7, 4);
}

#[test]
fn scan_string_unclosed() {
    let source = "\"abc";
    match scanner::scan(source, 1, 1) {
        Err((error_token, tokens)) => match error_token.token {
            Token::UnterminatedString(string) => {
                assert_eq!(tokens.len(), 0);
                assert_eq!(string, source.to_string());
                assert_eq!(error_token.line_no, 1);
                assert_eq!(error_token.col_no, 1);
                let new_source = source.to_string() + "\"";
                match scanner::scan(new_source.as_str(), 1, 1) {
                    Ok(tokens) => {
                        assert_eq!(tokens.len(), 2);
                        check_string_token(tokens.get(0), "abc", 1, 1, 3);
                        check_token(tokens.get(1), Token::Indent(0), 1, 6);
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


g (y) ->
    y
";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 16);

    // f
    check_token(tokens.get(0), Token::Identifier("f".to_string()), 1, 1);
    check_token(tokens.get(1), Token::LeftParen, 1, 3);
    check_token(tokens.get(2), Token::Identifier("x".to_string()), 1, 4);
    check_token(tokens.get(3), Token::RightParen, 1, 5);
    check_token(tokens.get(4), Token::FuncStart, 1, 7);
    check_token(tokens.get(5), Token::Indent(4), 2, 1);
    check_token(tokens.get(6), Token::Identifier("x".to_string()), 2, 5);
    check_token(tokens.get(7), Token::Indent(0), 3, 1);

    // g
    check_token(tokens.get(8), Token::Identifier("g".to_string()), 5, 1);
    check_token(tokens.get(9), Token::LeftParen, 5, 3);
    check_token(tokens.get(10), Token::Identifier("y".to_string()), 5, 4);
    check_token(tokens.get(11), Token::RightParen, 5, 5);
    check_token(tokens.get(12), Token::FuncStart, 5, 7);
    check_token(tokens.get(13), Token::Indent(4), 6, 1);
    check_token(tokens.get(14), Token::Identifier("y".to_string()), 6, 5);
    check_token(tokens.get(15), Token::Indent(0), 7, 1);
}

#[test]
#[should_panic]
fn scan_unknown() {
    let source = "{}";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 3);
    check_token(tokens.get(0), Token::Unknown('{'), 1, 1);
    check_token(tokens.get(1), Token::Unknown('}'), 1, 2);
    check_token(tokens.get(2), Token::Indent(0), 1, 3);
}

// Utilities -----------------------------------------------------------

/// Scan source and return tokens.
fn scan(source: &str) -> Vec<TokenWithPosition> {
    match scanner::scan(source, 1, 1) {
        Ok(tokens) => tokens,
        Err((error_token, tokens)) => panic!("Scan failed unexpectedly: {}", error_token),
    }
}

/// Check token returned by scanner against expected token.
fn check_token(actual: Option<&TokenWithPosition>, expected: Token, line_no: usize, col_no: usize) {
    assert_eq!(
        actual,
        Some(&TokenWithPosition::new(expected, line_no, col_no)),
    );
}

fn check_string_token(
    actual: Option<&TokenWithPosition>,
    expected_string: &str,
    expected_line_no: usize,
    expected_col_no: usize,
    expected_len: usize,
) {
    assert!(actual.is_some());
    match actual {
        Some(TokenWithPosition {
            token: Token::String(actual_string),
            line_no: actual_line_no,
            col_no: actual_col_no,
        }) => {
            assert_eq!(actual_string, expected_string);
            assert_eq!(actual_line_no, &expected_line_no);
            assert_eq!(actual_col_no, &expected_col_no);
            assert_eq!(actual_string.len(), expected_len);
        }
        _ => assert!(false),
    }
}
