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
