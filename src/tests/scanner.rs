use crate::scanner::{
    self, Location, ScanError, ScanErrorType, Token, TokenWithLocation,
};

#[test]
fn scan_empty() {
    let tokens = scan("");
    assert_eq!(tokens.len(), 0);
}

#[test]
fn scan_int() {
    let tokens = scan("123");
    assert_eq!(tokens.len(), 1);
    check_token(tokens.get(0), Token::Int("123".to_string(), 10), 1, 1, 1, 3);
}

#[test]
fn scan_binary_number() {
    let tokens = scan("0b11");
    assert_eq!(tokens.len(), 1);
    check_token(tokens.get(0), Token::Int("11".to_string(), 2), 1, 1, 1, 4);
}

#[test]
fn scan_float() {
    let tokens = scan("123.1");
    assert_eq!(tokens.len(), 1);
    check_token(tokens.get(0), Token::Float("123.1".to_string()), 1, 1, 1, 5);
}

#[test]
fn scan_float_with_e_and_no_sign() {
    let tokens = scan("123.1e1");
    assert_eq!(tokens.len(), 1);
    let expected = Token::Float("123.1E+1".to_string());
    check_token(tokens.get(0), expected, 1, 1, 1, 7);
}

#[test]
fn scan_float_with_e_and_sign() {
    let tokens = scan("123.1e+1");
    assert_eq!(tokens.len(), 1);
    let expected = Token::Float("123.1E+1".to_string());
    check_token(tokens.get(0), expected, 1, 1, 1, 8);
}

#[test]
fn scan_string_with_embedded_quote() {
    // "\"abc"
    let source = "\"\\\"abc\"";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 1);
    check_string_token(tokens.get(0), "\"abc", 1, 1, 1, 7);
}

#[test]
fn scan_string_with_newline() {
    // "abc
    // "
    let source = "\"abc\n\"";
    let tokens = scan(source);
    assert_eq!(tokens.len(), 1);
    check_string_token(tokens.get(0), "abc\n", 1, 1, 2, 1);
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
    assert_eq!(tokens.len(), 1);
    check_string_token(tokens.get(0), " a\nb\n\nc\n\n\n  ", 1, 1, 7, 3);
}

#[test]
fn scan_string_unclosed() {
    let source = "\"abc";
    match scanner::scan(source) {
        Err(err) => match err {
            ScanError {
                error: ScanErrorType::UnterminatedString(string),
                location,
            } => {
                assert_eq!(string, source.to_string());
                assert_eq!(location, Location::new(1, 1));
                let new_source = source.to_string() + "\"";
                match scanner::scan(new_source.as_str()) {
                    Ok(tokens) => {
                        assert_eq!(tokens.len(), 1);
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
f (x) ->  # 1
    x     # 2
    1     # 3
          # 4
          # 5
g (y) ->  # 6
    y     # 7\
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
    check_token(tokens.get(5), Token::BlockStart, 2, 0, 2, 0);
    token = Token::Identifier("x".to_string());
    check_token(tokens.get(6), token, 2, 5, 2, 5);
    check_token(tokens.get(7), Token::Int("1".to_string(), 10), 3, 5, 3, 5);
    check_token(tokens.get(8), Token::BlockEnd, 6, 0, 6, 0);

    // g
    token = Token::Identifier("g".to_string());
    check_token(tokens.get(9), token, 6, 1, 6, 1);
    check_token(tokens.get(10), Token::LeftParen, 6, 3, 6, 3);
    token = Token::Identifier("y".to_string());
    check_token(tokens.get(11), token, 6, 4, 6, 4);
    check_token(tokens.get(12), Token::RightParen, 6, 5, 6, 5);
    check_token(tokens.get(13), Token::FuncStart, 6, 7, 6, 8);
    check_token(tokens.get(14), Token::BlockStart, 7, 0, 7, 0);
    token = Token::Identifier("y".to_string());
    check_token(tokens.get(15), token, 7, 5, 7, 5);
    check_token(tokens.get(16), Token::BlockEnd, 8, 0, 8, 0);
    assert!(tokens.get(17).is_none());
}

#[test]
fn scan_unexpected_indent_on_first_line() {
    let source = "    abc = 1";
    match scanner::scan(source) {
        Ok(_) => assert!(false),
        Err(err) => match err {
            ScanError { error: ScanErrorType::UnexpectedIndent(1), location } => {
                assert_eq!(location.line, 1);
                assert_eq!(location.col, 1);
            }
            _ => assert!(false),
        },
    }
}

#[test]
fn scan_brackets() {
    let source = "

a = [
   1,
# comment
  2,
]

# FIXME: This is an unexpected indent but the scanner doesn't detect that.
    b = 1
";
    let tokens = scan(source);
    let mut token;
    for token in tokens {
        eprintln!("{}", token);
    }
    // assert_eq!(tokens.len(), 8);
    token = Token::Identifier("a".to_string());
    // check_token(tokens.get(0), token, 3, 1, 3, 1);
    // check_token(tokens.get(1), Token::Equal, 3, 3, 3, 3);
    // check_token(tokens.get(2), Token::LeftSquareBracket, 3, 5, 3, 5);
    // check_token(tokens.get(3), Token::Int("1".to_owned(), 10), 4, 4, 4, 4);
    // check_token(tokens.get(4), Token::Comma, 4, 5, 4, 5);
    // check_token(tokens.get(5), Token::Int("2".to_owned(), 10), 6, 3, 6, 3);
    // check_token(tokens.get(6), Token::Comma, 6, 4, 6, 4);
    // check_token(tokens.get(7), Token::RightSquareBracket, 7, 1, 7, 1);
    // assert!(tokens.get(8).is_none());
}

#[test]
fn scan_unknown() {
    let source = "{";
    match scanner::scan(source) {
        Ok(tokens) => assert!(false),
        Err(err) => match err {
            ScanError { error: ScanErrorType::UnknownToken(c), location } => {
                assert_eq!(c, '{');
                assert_eq!(location.line, 1);
                assert_eq!(location.col, 1);
            }
            _ => assert!(false),
        },
    }
}

// Utilities -----------------------------------------------------------

/// Scan source and return tokens.
fn scan(source: &str) -> Vec<TokenWithLocation> {
    match scanner::scan(source) {
        Ok(tokens) => tokens,
        Err(err) => panic!("Scan failed unexpectedly: {:?}", err),
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
            start: Location { line: actual_start_line, col: actual_start_col },
            end: Location { line: actual_end_line, col: actual_end_col },
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
