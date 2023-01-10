use crate::scanner::{ScanTokensResult, Scanner, TokenWithLocation};
use crate::util::source_from_text;

#[derive(Clone, Debug, PartialEq)]
pub enum FormatStrToken {
    Str(String),
    Expr(Vec<TokenWithLocation>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FormatStrErr {
    EmptyExpr(usize),
    UnmatchedOpeningBracket(usize),
    UnmatchedClosingBracket(usize),
    ScanErr(usize, usize),
}

/// Split format string into string and expression parts. Expression
/// parts will be scanned by the main scanner into tokens.
///
/// The default delimiters are "{" and "}" but any other delimiter can
/// be used, e.g "{{" and "}}" or "${" and "}".
pub fn scan_format_string(
    string: &str,
    delimiters: Option<(&str, &str)>,
) -> Result<Vec<FormatStrToken>, FormatStrErr> {
    use FormatStrErr::*;
    use FormatStrToken::*;

    let len = string.len();
    let (open, close) =
        if let Some(deliminators) = delimiters { deliminators } else { ("{", "}") };
    let open_delim_len = open.len();
    let close_delim_len = close.len();

    let mut tokens: Vec<FormatStrToken> = Vec::new();
    let mut stack = vec![];

    // Current position in string
    let mut pos = 0usize;

    // Accumulator for current string/non-expression part. This is
    // needed in order to skip over backslashes used to escape format
    // string brackets.
    let mut str = String::with_capacity(32);

    while pos < len {
        let current_char = &string[pos..pos + 1];

        let open_end = pos + open_delim_len;
        let open_slice = if open_end > len { "" } else { &string[pos..open_end] };

        let close_end = pos + close_delim_len;
        let close_slice = if close_end > len { "" } else { &string[pos..close_end] };

        // Escaped brackets are handled as literals
        let escaped = str.ends_with('\\');

        if escaped && (open_slice == open || close_slice == close) {
            str.pop();
            str.push_str(current_char);
            pos += 1;
        } else if open_slice == open {
            // Stack entry points at first char of open delimiter
            stack.push(pos);
            if !str.is_empty() {
                tokens.push(Str(str.clone()));
                str.clear();
            }
            pos = open_end;
        } else if close_slice == close {
            if let Some(open_pos) = stack.pop() {
                if stack.is_empty() {
                    let expr_start_pos = open_pos + open_delim_len;
                    let expr = string[expr_start_pos..pos].trim();
                    if expr.is_empty() {
                        return Err(EmptyExpr(open_pos));
                    }
                    let mut source = source_from_text(expr);
                    let scanner = Scanner::new(&mut source);
                    let result: ScanTokensResult = scanner.collect();
                    match result {
                        Ok(expr_tokens) => tokens.push(Expr(expr_tokens)),
                        Err(_) => return Err(ScanErr(open_pos + open_delim_len, pos)),
                    }
                }
            } else {
                return Err(UnmatchedClosingBracket(pos));
            }
            pos = close_end;
        } else {
            if stack.is_empty() {
                str.push_str(current_char);
            }
            pos += 1;
        }
    }

    if !stack.is_empty() {
        return Err(UnmatchedOpeningBracket(stack.pop().unwrap()));
    }

    // Include trailing string/non-expression part
    if !str.is_empty() {
        tokens.push(Str(str.clone()));
    }

    Ok(tokens)
}
