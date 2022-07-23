use crate::scanner::{ScanTokensResult, Scanner, TokenWithLocation};
use crate::util::source_from_text;

#[derive(Clone, Debug, PartialEq)]
pub enum FormatStrToken {
    Str(String),
    Expr(Vec<TokenWithLocation>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum FormatStrErr {
    EmptyExpr(usize),
    UnmatchedOpeningBracket(usize),
    UnmatchedClosingBracket(usize),
    ScanErr(usize, usize),
}

pub fn scan_format_string(string: &str) -> Result<Vec<FormatStrToken>, FormatStrErr> {
    use FormatStrErr::*;
    use FormatStrToken::*;

    let mut tokens: Vec<FormatStrToken> = Vec::new();
    let mut chars = string.chars();
    let mut peek_chars = string.chars();
    let mut stack = vec![];

    // Current position in string
    let mut pos = 0usize;

    // Accumulator for current string/non-expression part. This is
    // needed in order to skip over backslashes used to escape format
    // string brackets.
    let mut str = String::with_capacity(32);

    peek_chars.next();

    while let Some(c) = chars.next() {
        let d = peek_chars.next();
        match (c, d) {
            ('\\', Some(d @ ('{' | '}'))) => {
                // Escaped brackets are handled as literals
                str.push(d);
                chars.next();
                peek_chars.next();
                pos += 1;
            }
            ('{', _) => {
                // Start of expression
                stack.push(pos);
                if str.len() > 0 {
                    tokens.push(Str(str.clone()));
                    str.clear();
                }
            }
            ('}', _) => {
                // End of expression
                if let Some(i) = stack.pop() {
                    if stack.len() == 0 {
                        let expr = string[i + 1..pos].trim();
                        if expr.len() == 0 {
                            return Err(EmptyExpr(i));
                        }
                        let mut source = source_from_text(expr);
                        let scanner = Scanner::new(&mut source);
                        let result: ScanTokensResult = scanner.collect();
                        match result {
                            Ok(expr_tokens) => tokens.push(Expr(expr_tokens)),
                            Err(_) => return Err(ScanErr(i, pos)),
                        }
                    }
                } else {
                    return Err(UnmatchedClosingBracket(pos));
                }
            }
            _ => {
                if stack.len() == 0 {
                    str.push(c);
                }
            }
        }
        pos += 1;
    }

    if stack.len() > 0 {
        return Err(UnmatchedOpeningBracket(stack.pop().unwrap()));
    }

    if str.len() > 0 {
        tokens.push(Str(str.clone()));
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;

    use crate::scanner::Token;
    use crate::util::Location;

    use super::FormatStrErr::*;
    use super::FormatStrToken::*;
    use super::*;

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

        token = tokens.get(4).unwrap();
        let expected = Str("c".to_owned());
        assert_eq!(token, &expected);
    }

    #[test]
    fn scan_complex() {
        let tokens = scan_ok("aaa{1 + 1}bbb{2 + 2}ccc{$'{3 + 3}xxx{4 + 4}'}ddd", 7);
        // TODO: Check tokens
    }

    #[test]
    fn scan_with_tuple() {
        let tokens = scan_ok("{(1, 2, 3, 'a', 'b', 'c')}", 1);
        let token = tokens.last().unwrap();
        // TODO: Check tokens
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
        assert_eq!(result, Err(UnmatchedOpeningBracket(0)));
        let result = scan_format_string("a{1");
        assert_eq!(result, Err(UnmatchedOpeningBracket(1)));
    }

    #[test]
    fn scan_unmatched_closing_bracket() {
        let result = scan_format_string("1}");
        assert_eq!(result, Err(UnmatchedClosingBracket(1)));
        let result = scan_format_string("a1}");
        assert_eq!(result, Err(UnmatchedClosingBracket(2)));
    }
}
