use crate::scanner::{Scanner, TokenWithPosition};
use crate::tokens::Token;

#[test]
fn new() {
    let source = "";
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan(true).unwrap();
    assert_eq!(tokens.len(), 1);
    check_end_of_input(source, tokens);
}

#[test]
fn scan_string_with_embedded_quote() {
    let source = "\"\\\"abc\"";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 2);
    check_token(tokens.get(0), Token::String("\"abc".to_string()), 1, 1, 7);
    check_end_of_input(source, tokens);
}

#[test]
fn scan_string_unclosed() {
    let source = "\"abc";
    let mut scanner = Scanner::new(source);
    let tokens = match scanner.scan(false) {
        Ok(tokens) => tokens,
        _ => vec![],
    };
    let actual = tokens.get(0).unwrap();
    let expected = Token::NeedsMoreInput("\"abc".to_string());
    assert_eq!(tokens.len(), 1);
    check_token(Some(actual), expected, 1, 1, 4);
    scanner.add_source(format!("{}\"", actual.token.0).as_str());
    scanner.scan(true);
    // check_token(tokens.get(0), Token::String("abc".to_string()), 1, 1, 4);
    // check_end_of_input(source, tokens);
}

#[test]
#[should_panic]
fn scan_unknown() {
    let source = "{}";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 3);
    check_token(tokens.get(0), Token::Unknown('{'), 1, 1, 1);
    check_token(tokens.get(1), Token::Unknown('}'), 1, 2, 1);
    check_end_of_input(source, tokens);
}

/// Scan source and return tokens.
fn scan(source: &str) -> Vec<TokenWithPosition> {
    let mut scanner = Scanner::new(source);
    match scanner.scan(true) {
        Ok(tokens) => tokens,
        Err(message) => panic!("Scan failed unexpectedly: {}", message),
    }
}

/// Check token returned by scanner against expected token.
fn check_token(
    actual: Option<&TokenWithPosition>,
    token: Token,
    line_no: usize,
    col_no: usize,
    length: usize,
) {
    assert_eq!(
        actual,
        Some(&TokenWithPosition::new(token, line_no, col_no, length)),
    );
}

/// Ensure the last token is end-of-input.
fn check_end_of_input(source: &str, tokens: Vec<TokenWithPosition>) {
    check_token(
        tokens.get(tokens.len() - 1),
        Token::EndOfInput,
        1,
        source.len() + 1,
        0,
    );
}
