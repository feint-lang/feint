use num_bigint::BigInt;

use crate::source::{source_from_text, Location};

use crate::scanner::*;

/// Create a scanner from the specified text, scan the text, and return
/// the resulting tokens or error.
pub fn scan_text(text: &str) -> ScanTokensResult {
    let mut source = source_from_text(text);
    let scanner = Scanner::new(&mut source);
    scanner.collect()
}

/// Scan text and assume success, returning tokens in unwrapped form.
/// Panic on error. Mainly useful for testing.
pub fn scan_optimistic(text: &str) -> Vec<TokenWithLocation> {
    match scan_text(text) {
        Ok(tokens) => tokens,
        Err(err) => panic!("Scan failed unexpectedly: {:?}", err),
    }
}

/// Scan text and assume success, returning inner tokens.
/// Panic on error. Mainly useful for testing.
pub fn scan_to_tokens(text: &str) -> Vec<Token> {
    let tokens = scan_optimistic(text);
    let tokens: Vec<Token> = tokens.iter().map(|t| t.token.clone()).collect();
    tokens
}

#[test]
fn scan_empty() {
    let tokens = scan_optimistic("");
    assert_eq!(tokens.len(), 0);
}

#[test]
fn scan_int() {
    let tokens = scan_optimistic("123");
    assert_eq!(tokens.len(), 2);
    check_token(tokens.get(0), Token::Int(BigInt::from(123)), 1, 1, 1, 3);
    check_token(tokens.get(1), Token::EndOfStatement, 1, 4, 1, 4);
}

#[test]
fn scan_binary_number() {
    let tokens = scan_optimistic("0b11"); // = 3
    assert_eq!(tokens.len(), 2);
    check_token(tokens.get(0), Token::Int(BigInt::from(3)), 1, 1, 1, 4);
    check_token(tokens.get(1), Token::EndOfStatement, 1, 5, 1, 5);
}

#[test]
fn scan_float() {
    let tokens = scan_optimistic("123.1");
    assert_eq!(tokens.len(), 2);
    check_token(tokens.get(0), Token::Float(123.1_f64), 1, 1, 1, 5);
    check_token(tokens.get(1), Token::EndOfStatement, 1, 6, 1, 6);
}

#[test]
fn scan_float_with_e_and_no_sign() {
    let tokens = scan_optimistic("123.1e1");
    assert_eq!(tokens.len(), 2);
    let expected = Token::Float(123.1E+1);
    check_token(tokens.get(0), expected, 1, 1, 1, 7);
    check_token(tokens.get(1), Token::EndOfStatement, 1, 8, 1, 8);
}

#[test]
fn scan_float_with_e_and_sign() {
    let tokens = scan_optimistic("123.1e+1");
    assert_eq!(tokens.len(), 2);
    let expected = Token::Float(123.1E+1);
    check_token(tokens.get(0), expected, 1, 1, 1, 8);
    check_token(tokens.get(1), Token::EndOfStatement, 1, 9, 1, 9);
}

#[test]
fn scan_string_with_embedded_quote() {
    // "\"abc"
    let source = "\"\\\"abc\"";
    let tokens = scan_optimistic(source);
    assert_eq!(tokens.len(), 2);
    check_string_token(tokens.get(0), "\"abc", 1, 1, 1, 7);
    check_token(tokens.get(1), Token::EndOfStatement, 1, 8, 1, 8);
}

#[test]
fn scan_string_with_newline() {
    // "abc
    // "
    let source = "\"abc\n\"";
    let tokens = scan_optimistic(source);
    assert_eq!(tokens.len(), 2);
    check_string_token(tokens.get(0), "abc\n", 1, 1, 2, 1);
    check_token(tokens.get(1), Token::EndOfStatement, 2, 2, 2, 2);
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
    let tokens = scan_optimistic(source);
    assert_eq!(tokens.len(), 2);
    check_string_token(tokens.get(0), " a\nb\n\nc\n\n\n  ", 1, 1, 7, 3);
    check_token(tokens.get(1), Token::EndOfStatement, 7, 4, 7, 4);
}

#[test]
fn scan_string_with_escaped_chars() {
    let tokens = scan_optimistic("\"\\0\\a\\b\\n\\'\\\"\"");
    assert_eq!(tokens.len(), 2);
    // NOTE: We could put a backslash before the single quote in
    //       the expected string, but Rust seems to treat \' and '
    //       as the same.
    check_string_token(tokens.get(0), "\0\x07\x08\n'\"", 1, 1, 1, 14);
    check_token(tokens.get(1), Token::EndOfStatement, 1, 15, 1, 15);
}

#[test]
fn scan_string_with_escaped_regular_char() {
    let tokens = scan_optimistic("\"ab\\c\"");
    assert_eq!(tokens.len(), 2);
    check_string_token(tokens.get(0), "ab\\c", 1, 1, 1, 6);
    check_token(tokens.get(1), Token::EndOfStatement, 1, 7, 1, 7);
}

#[test]
fn scan_string_unclosed() {
    let items = vec![
        ("\"abc", "abc", 1, 1, 1, 5),
        ("\"abc\n", "abc\n", 1, 1, 2, 1),
        ("\"abc\n\n", "abc\n\n", 1, 1, 3, 1),
    ];
    for (source, expected, l1, c1, l2, c2) in items {
        match scan_text(source) {
            Err(err) => match err {
                ScanErr { kind: ScanErrKind::UnterminatedStr(string), location } => {
                    assert_eq!(string, source.to_string());
                    assert_eq!(location, Location::new(1, 1));
                    let new_source = source.to_string() + "\"";
                    match scan_text(new_source.as_str()) {
                        Ok(tokens) => {
                            assert_eq!(tokens.len(), 2);
                            check_string_token(tokens.get(0), expected, l1, c1, l2, c2);
                        }
                        _ => assert!(false),
                    }
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        }
    }
}

#[test]
fn scan_indents() {
    let source = "\
f (x) =>  # 1
    x     # 2
    1     # 3
          # 4
          # 5
g (y) =>  # 6
    y     # 7\
";
    let tokens = scan_optimistic(source);
    let mut tokens = tokens.iter();

    // f
    check_token(tokens.next(), Token::Ident("f".to_string()), 1, 1, 1, 1);
    check_token(tokens.next(), Token::LParen, 1, 3, 1, 3);
    check_token(tokens.next(), Token::Ident("x".to_string()), 1, 4, 1, 4);
    check_token(tokens.next(), Token::RParen, 1, 5, 1, 5);
    check_token(tokens.next(), Token::FuncScopeStart, 1, 7, 1, 8);
    check_token(tokens.next(), Token::Ident("x".to_string()), 2, 5, 2, 5);
    check_token(tokens.next(), Token::EndOfStatement, 2, 6, 2, 6);
    check_token(tokens.next(), Token::Int(BigInt::from(1)), 3, 5, 3, 5);
    check_token(tokens.next(), Token::EndOfStatement, 3, 6, 3, 6);
    check_token(tokens.next(), Token::ScopeEnd, 4, 1, 4, 1);
    check_token(tokens.next(), Token::EndOfStatement, 4, 1, 4, 1);

    // g
    check_token(tokens.next(), Token::Ident("g".to_string()), 6, 1, 6, 1);
    check_token(tokens.next(), Token::LParen, 6, 3, 6, 3);
    check_token(tokens.next(), Token::Ident("y".to_string()), 6, 4, 6, 4);
    check_token(tokens.next(), Token::RParen, 6, 5, 6, 5);
    check_token(tokens.next(), Token::FuncScopeStart, 6, 7, 6, 8);
    check_token(tokens.next(), Token::Ident("y".to_string()), 7, 5, 7, 5);
    check_token(tokens.next(), Token::EndOfStatement, 7, 6, 7, 6);
    check_token(tokens.next(), Token::ScopeEnd, 8, 1, 8, 1);
    check_token(tokens.next(), Token::EndOfStatement, 8, 1, 8, 1);

    assert!(tokens.next().is_none());
}

#[test]
fn scan_unexpected_indent_on_first_line() {
    let source = "    abc = 1";
    let result = scan_text(source);
    assert!(result.is_err());
    match result.unwrap_err() {
        ScanErr { kind: ScanErrKind::UnexpectedIndent(1), location } => {
            assert_eq!(location.line, 1);
            assert_eq!(location.col, 1);
        }
        err => assert!(false, "Unexpected error: {:?}", err),
    }
}

#[test]
fn scan_brackets() {
    let source = "

a = [
   1,
# comment
  2,
]  # another comment

b = 3
";
    let tokens = scan_optimistic(source);
    let mut tokens = tokens.iter();

    check_token(tokens.next(), Token::Ident("a".to_string()), 3, 1, 3, 1);
    check_token(tokens.next(), Token::Equal, 3, 3, 3, 3);
    check_token(tokens.next(), Token::LBracket, 3, 5, 3, 5);
    check_token(tokens.next(), Token::Int(BigInt::from(1)), 4, 4, 4, 4);
    check_token(tokens.next(), Token::Comma, 4, 5, 4, 5);
    check_token(tokens.next(), Token::Int(BigInt::from(2)), 6, 3, 6, 3);
    check_token(tokens.next(), Token::Comma, 6, 4, 6, 4);
    check_token(tokens.next(), Token::RBracket, 7, 1, 7, 1);
    check_token(tokens.next(), Token::EndOfStatement, 7, 2, 7, 2);
    check_token(tokens.next(), Token::Ident("b".to_string()), 9, 1, 9, 1);
    check_token(tokens.next(), Token::Equal, 9, 3, 9, 3);
    check_token(tokens.next(), Token::Int(BigInt::from(3)), 9, 5, 9, 5);
    check_token(tokens.next(), Token::EndOfStatement, 9, 6, 9, 6);
    assert!(tokens.next().is_none());
}

#[test]
fn scan_unknown() {
    let source = "\\";
    match scan_text(source) {
        Ok(_tokens) => assert!(false),
        Err(err) => match err {
            ScanErr { kind: ScanErrKind::UnexpectedChar(c), location } => {
                assert_eq!(c, '\\');
                assert_eq!(location.line, 1);
                assert_eq!(location.col, 1);
            }
            _ => assert!(false),
        },
    }
}

#[test]
fn scan_inline_block_simple() {
    use Token::*;
    let tokens = scan_to_tokens("block -> true");
    assert_eq!(
        tokens,
        vec![
            Block,
            InlineScopeStart,
            True,
            EndOfStatement,
            InlineScopeEnd,
            EndOfStatement
        ]
    );
}

#[test]
fn scan_inline_block_simple_in_parens() {
    use Token::*;
    let tokens = scan_to_tokens("(block -> true)");
    assert_eq!(
        tokens,
        vec![
            LParen,
            Block,
            InlineScopeStart,
            True,
            EndOfStatement,
            InlineScopeEnd,
            RParen,
            EndOfStatement,
        ]
    );
}

#[test]
fn scan_inline_block_if_else() {
    use Token::*;
    let tokens = scan_to_tokens("if true -> (1,) else -> (2,)");
    assert_eq!(
        tokens,
        vec![
            If,
            True,
            InlineScopeStart,
            LParen,
            Int(BigInt::from(1)),
            Comma,
            RParen,
            EndOfStatement,
            InlineScopeEnd,
            EndOfStatement,
            Else,
            InlineScopeStart,
            LParen,
            Int(BigInt::from(2)),
            Comma,
            RParen,
            EndOfStatement,
            InlineScopeEnd,
            EndOfStatement,
        ]
    );
}

#[test]
fn scan_inline_block_1() {
    use Token::*;
    let tokens = scan_to_tokens("block -> ()");
    assert_eq!(
        tokens,
        vec![
            Block,
            InlineScopeStart,
            LParen,
            RParen,
            EndOfStatement,
            InlineScopeEnd,
            EndOfStatement
        ]
    );
}

#[test]
fn scan_inline_block_2() {
    use Token::*;
    let tokens = scan_to_tokens("(block -> 1, block -> 2)");
    assert_eq!(
        tokens,
        vec![
            LParen,
            Block,
            InlineScopeStart,
            Int(BigInt::from(1)),
            EndOfStatement,
            InlineScopeEnd,
            Comma,
            Block,
            InlineScopeStart,
            Int(BigInt::from(2)),
            EndOfStatement,
            InlineScopeEnd,
            RParen,
            EndOfStatement,
        ]
    );
}

#[test]
fn scan_inline_block_3() {
    use Token::*;
    let tokens = scan_to_tokens("block -> (1, 2)");
    assert_eq!(
        tokens,
        vec![
            Block,
            InlineScopeStart,
            LParen,
            Int(BigInt::from(1)),
            Comma,
            Int(BigInt::from(2)),
            RParen,
            EndOfStatement,
            InlineScopeEnd,
            EndOfStatement,
        ]
    );
}

#[test]
fn scan_inline_block_4_without_parens() {
    use Token::*;
    let tokens = scan_to_tokens("block -> block -> true");
    assert_eq!(
        tokens,
        vec![
            Block,
            InlineScopeStart,
            Block,
            InlineScopeStart,
            True,
            EndOfStatement,
            InlineScopeEnd,
            EndOfStatement,
            InlineScopeEnd,
            EndOfStatement,
        ]
    );
}

#[test]
fn scan_inline_block_5_with_parens() {
    use Token::*;
    let tokens = scan_to_tokens("(block -> block -> true)");
    assert_eq!(
        tokens,
        vec![
            LParen,
            Block,
            InlineScopeStart,
            Block,
            InlineScopeStart,
            True,
            EndOfStatement,
            InlineScopeEnd,
            EndOfStatement,
            InlineScopeEnd,
            RParen,
            EndOfStatement,
        ]
    );
}

// Utilities -------------------------------------------------------

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
            token: Token::Str(actual_string),
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
