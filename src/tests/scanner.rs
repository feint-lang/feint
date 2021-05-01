use crate::scanner::{Scanner, TokenWithPosition};
use crate::tokens::Token;

#[test]
fn new() {
    let source = "";
    let mut scanner = Scanner::new();
    let tokens = scanner.scan(source).unwrap();
    assert_eq!(tokens.len(), 1);
    check_token(tokens.last(), Token::EndOfInput, 1, 1, 0);
}

#[test]
fn scan_string_with_embedded_quote() {
    let source = "\"\\\"abc\"";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 2);
    check_token(tokens.get(0), Token::String("\"abc".to_string()), 1, 1, 7);
    check_token(tokens.last(), Token::EndOfInput, 1, 8, 0);
}

#[test]
fn scan_string_with_newline() {
    let source = "\"abc\n\"";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 2);
    check_token(tokens.get(0), Token::String("abc\n".to_string()), 1, 1, 6);
    check_token(tokens.last(), Token::EndOfInput, 2, 2, 0);
}

#[test]
fn scan_string_with_many_newlines() {
    //   " a
    // b
    //
    // c
    //
    //
    //   "
    let source = "  \" a\nb\n\nc\n\n\n  \"";
    let tokens = scan(source);
    let expected_string = " a\nb\n\nc\n\n\n  ".to_string();
    assert_eq!(tokens.len(), 3);
    // Overall length includes quotes
    check_token(tokens.get(0), Token::Whitespace("  ".to_string()), 1, 1, 2);
    check_token(tokens.get(1), Token::String(expected_string), 1, 3, 14);
    check_token(tokens.last(), Token::EndOfInput, 7, 4, 0);
}

#[test]
fn scan_string_unclosed() {
    let source = "\"abc";
    let mut scanner = Scanner::new();
    match scanner.scan(source) {
        Err((error_token, tokens)) => match error_token.token {
            Token::NeedsMoreInput(remaining_input) => {
                assert_eq!(tokens.len(), 0);
                assert_eq!(remaining_input, source.to_string());
                assert_eq!(error_token.line_no, 1);
                assert_eq!(error_token.col_no, 1);
                assert_eq!(error_token.length, 4);
                let input = format!("{}\"", remaining_input);
                let new_source = input.as_str();
                match scanner.scan(input.as_str()) {
                    Ok(tokens) => {
                        // 1 string, 1 end-of-input
                        assert_eq!(tokens.len(), 2);
                        // Overall length includes the quote chars
                        check_token(tokens.get(0), Token::String("abc".to_string()), 1, 1, 5);
                        check_token(tokens.last(), Token::EndOfInput, 1, 6, 0);
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
#[should_panic]
fn scan_unknown() {
    let source = "{}";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 3);
    check_token(tokens.get(0), Token::Unknown('{'), 1, 1, 1);
    check_token(tokens.get(1), Token::Unknown('}'), 1, 2, 1);
    check_token(tokens.last(), Token::EndOfInput, 1, 3, 0);
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
fn check_token(
    actual: Option<&TokenWithPosition>,
    expected: Token,
    line_no: usize,
    col_no: usize,
    length: usize,
) {
    assert_eq!(
        actual,
        Some(&TokenWithPosition::new(expected, line_no, col_no, length)),
    );
}
