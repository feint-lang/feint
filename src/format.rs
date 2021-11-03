#[derive(Debug, PartialEq)]
pub struct Span(usize, usize);

#[derive(Debug, PartialEq)]
pub enum Token {
    // string not inside ${} group, start, end
    String(String, Span),
    // string inside ${} group, start, end
    Group(String, Span),
}

#[derive(Debug, PartialEq)]
pub enum FormatStringErr {
    EmptyGroup(Span),
    UnclosedGroup(Span),
}

pub fn scan(string: &str) -> Result<Vec<Token>, FormatStringErr> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = string.chars();
    let mut peek_chars = string.chars();

    // Current position in format string
    let mut pos = 0usize;

    // Current non-group part
    let mut part = String::with_capacity(32);

    // Current expression inside group
    let mut expr = String::with_capacity(32);

    peek_chars.next();

    while let Some(c) = chars.next() {
        let d = peek_chars.next();

        match (c, d) {
            ('\\', Some('$')) => {
                chars.next();
                peek_chars.next();
                part.push('$');
                pos += 1;
            }

            ('$', Some('{')) => {
                if part.len() > 0 {
                    let (start, end) = (pos - part.len(), pos - 1);
                    tokens.push(Token::String(part.clone(), Span(start, end)));
                    part.clear();
                }

                chars.next();
                peek_chars.next();
                pos += 1;

                while let Some(c) = chars.next() {
                    peek_chars.next();
                    pos += 1;

                    if c == '}' {
                        let (start, end) = (pos - expr.len() - 2, pos);
                        let span = Span(start, end);
                        let trimmed = expr.trim();
                        if trimmed.len() == 0 {
                            return Err(FormatStringErr::EmptyGroup(span));
                        }
                        tokens.push(Token::Group(trimmed.to_owned(), span));
                        expr.clear();
                        break;
                    }

                    expr.push(c);
                }
            }

            _ => {
                part.push(c);
            }
        }

        pos += 1;
    }

    if expr.len() > 0 {
        return Err(FormatStringErr::UnclosedGroup(Span(
            pos - expr.len() - 2,
            pos - 1,
        )));
    }

    if part.len() > 0 {
        let (start, end) = (pos - part.len(), pos - 1);
        tokens.push(Token::String(part.clone(), Span(start, end)));
        part.clear();
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::FormatStringErr;

    fn scan_ok(string: &str, expected_num_tokens: usize) -> Vec<Token> {
        let result = scan(string);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert_eq!(tokens.len(), expected_num_tokens);
        tokens
    }

    #[test]
    fn scan_simple() {
        let tokens = scan_ok("${1}", 1);
        let token = tokens.last().unwrap();
        let expected = Token::Group("1".to_owned(), Span(0, 3));
        assert_eq!(token, &expected);
    }

    #[test]
    fn scan_two_groups() {
        let tokens = scan_ok("a${1}b${'2'}c", 5);
        let mut token;
        token = tokens.get(0).unwrap();
        let expected = Token::String("a".to_owned(), Span(0, 0));
        assert_eq!(token, &expected);
        token = tokens.get(1).unwrap();
        let expected = Token::Group("1".to_owned(), Span(1, 4));
        assert_eq!(token, &expected);
        token = tokens.get(2).unwrap();
        let expected = Token::String("b".to_owned(), Span(5, 5));
        assert_eq!(token, &expected);
        token = tokens.get(3).unwrap();
        let expected = Token::Group("'2'".to_owned(), Span(6, 11));
        assert_eq!(token, &expected);
        token = tokens.get(4).unwrap();
        let expected = Token::String("c".to_owned(), Span(12, 12));
        assert_eq!(token, &expected);
    }

    #[test]
    fn scan_with_tuple() {
        let tokens = scan_ok("${(1, 2, 3, 'a', 'b', 'c')}", 1);
        let token = tokens.last().unwrap();
        let expected = Token::Group("(1, 2, 3, 'a', 'b', 'c')".to_owned(), Span(0, 26));
        assert_eq!(token, &expected);
    }

    #[test]
    fn scan_escaped_dollar() {
        let tokens = scan_ok("\\${}", 1);
        let token = tokens.last().unwrap();
        let expected = Token::String("${}".to_owned(), Span(1, 3));
        assert_eq!(token, &expected);
    }

    #[test]
    fn scan_no_groups() {
        let tokens = scan_ok("abc", 1);
        let token = tokens.last().unwrap();
        let expected = Token::String("abc".to_owned(), Span(0, 2));
        assert_eq!(token, &expected);
    }

    #[test]
    fn scan_empty_group() {
        let result = scan("${}");
        assert_eq!(result, Err(FormatStringErr::EmptyGroup(Span(0, 2))))
    }

    #[test]
    fn scan_unclosed_groups() {
        let result = scan("${1");
        assert_eq!(result, Err(FormatStringErr::UnclosedGroup(Span(0, 2))));
        let result = scan("a${1");
        assert_eq!(result, Err(FormatStringErr::UnclosedGroup(Span(1, 3))));
    }
}
