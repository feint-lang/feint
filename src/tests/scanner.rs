use crate::scanner::{Scanner, TokenWithPosition};
use crate::tokens::Token;

#[test]
fn new() {
    let source = "";
    let mut scanner = Scanner::new();
    let tokens = scanner.scan(source).unwrap();
    assert_eq!(tokens.len(), 0);
}

#[test]
fn scan_string_with_embedded_quote() {
    // "\"abc"
    let source = "\"\\\"abc\"";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 2);
    check_token(tokens.get(0), Token::String("\"abc".to_string()), 1, 1);
    check_token(tokens.last(), Token::Indent(0), 1, 8);
}

#[test]
fn scan_string_with_newline() {
    // "abc
    // "
    let source = "\"abc\n\"";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 2);
    check_token(tokens.get(0), Token::String("abc\n".to_string()), 1, 1);
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
    let expected_string = " a\nb\n\nc\n\n\n  ".to_string();
    assert_eq!(tokens.len(), 2);
    check_token(tokens.get(0), Token::String(expected_string), 1, 1);
    check_token(tokens.get(1), Token::Indent(0), 7, 4);
}

#[test]
fn scan_string_unclosed() {
    let source = "\"abc";
    let mut scanner = Scanner::new();
    match scanner.scan(source) {
        Err((error_token, tokens)) => match error_token.token {
            Token::UnterminatedString(string) => {
                assert_eq!(tokens.len(), 0);
                assert_eq!(string, source.to_string());
                assert_eq!(error_token.line_no, 1);
                assert_eq!(error_token.col_no, 1);
                let new_input = string + "\"";
                match scanner.scan(new_input.as_str()) {
                    Ok(tokens) => {
                        assert_eq!(tokens.len(), 2);
                        check_token(tokens.get(0), Token::String("abc".to_string()), 1, 1);
                        check_token(tokens.last(), Token::Indent(0), 1, 6);
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
    check_token(tokens.last(), Token::Indent(0), 1, 3);
}

/// Scan source and return tokens.
fn scan(source: &str) -> Vec<TokenWithPosition> {
    let mut scanner = Scanner::new();
    match scanner.scan(source) {
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
    expected_line_no: usize,
    expected_col_no: usize,
    expected_length: usize,
) {
    assert!(actual.is_some());
    match actual {
        Some(TokenWithPosition {
            token: Token::String(string),
            line_no: expected_line_no,
            col_no: expected_col_no,
        }) => {
            assert_eq!(string.len(), expected_length);
        }
        _ => assert!(false),
    }
}
