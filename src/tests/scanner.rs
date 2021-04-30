use crate::scanner::{Scanner, TokenWithPosition};
use crate::tokens::Token;

#[test]
fn new() {
    let source = "";
    let mut scanner = Scanner::new();
    let tokens = scanner.scan(source, true).unwrap();
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
    let mut scanner = Scanner::new();

    // Scan the unclosed string, which should return a "needs more
    // input" token.

    let mut tokens = match scanner.scan(source, false) {
        Ok(tokens) => tokens,
        _ => vec![],
    };

    let actual = tokens.get(0);
    let expected = Token::NeedsMoreInput(source.to_string());
    assert_eq!(tokens.len(), 1);
    check_token(actual, expected, 1, 1, 4);

    // Scan the closing quote. Now a string token should be returned.

    match tokens.last() {
        Some(TokenWithPosition {
            token: Token::NeedsMoreInput(remaining),
            line_no: _,
            col_no: _,
            length: _,
        }) => {
            let more_source = format!("{}\"", remaining);
            let more_tokens = match scanner.scan(more_source.as_str(), true) {
                Ok(tokens) => tokens,
                _ => vec![],
            };
            let more_actual = more_tokens.get(0);
            let more_expected = Token::String("abc".to_string());

            assert_eq!(more_tokens.len(), 2);
            check_token(more_actual, more_expected, 1, 1, 5);
            check_end_of_input(more_source.as_str(), more_tokens);
        }
        _ => (),
    }
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
    let mut scanner = Scanner::new();
    match scanner.scan(source, true) {
        Ok(tokens) => tokens,
        Err(message) => panic!("Scan failed unexpectedly: {}", message),
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

/// Ensure the last token is end-of-input.
fn check_end_of_input(source: &str, tokens: Vec<TokenWithPosition>) {
    check_token(tokens.last(), Token::EndOfInput, 1, source.len() + 1, 0);
}
