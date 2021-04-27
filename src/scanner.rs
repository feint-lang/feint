use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

use crate::tokens::Token;
use std::process::id;

pub struct Scanner<'a> {
    /// Stream of input characters from input string
    stream: Peekable<Chars<'a>>,
    /// The same stream but one character ahead for easier lookaheads
    lookahead_stream: Peekable<Chars<'a>>,
    line_no: usize,
    col_no: usize,
}

#[derive(Debug)]
pub struct TokenWithPosition {
    token: Token,
    line_no: usize,
    col_no: usize,
    length: usize,
}

impl fmt::Display for TokenWithPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token {}:{} {:?}", self.line_no, self.col_no, self.token)
    }
}

impl<'a> Scanner<'a> {
    pub fn new(source: &str) -> Scanner {
        let stream = source.chars().peekable();
        let mut peek_stream = source.chars().peekable();
        let line_no = 1;
        let col_no = 1;
        peek_stream.next();
        Scanner { stream, lookahead_stream: peek_stream, line_no, col_no }
    }

    pub fn scan(&mut self) -> Vec<TokenWithPosition> {
        let mut tokens: Vec<TokenWithPosition> = vec!();
        loop {
            let token = self.next_token();
            if token.token == Token::Eof {
                tokens.push(token);
                break;
            } else {
                tokens.push(token);
            }
            self.skip_whitespace();
        }
        tokens
    }

    fn next_token(&mut self) -> TokenWithPosition {
        let line_no = self.line_no;
        let col_no = self.col_no;

        let token = match self.next() {
            Some(('"', _)) => Token::String(self.read_string('"')),
            Some(('#', _)) => Token::Comment(self.read_comment()),
            Some(('(', _)) => Token::LeftParen,
            Some((')', _)) => Token::RightParen,
            Some(('[', _)) => Token::LeftSquareBracket,
            Some((']', _)) => Token::RightSquareBracket,
            Some(('<', d)) => match d {
                Some('=') => Token::LessThanOrEqual,
                Some('-') => Token::LoopFeed,
                _ => Token::LeftAngleBracket,
            },
            Some(('>', d)) => match d {
                Some('=') => Token::GreaterThanOrEqual,
                _ => Token::RightAngleBracket,
            },
            Some(('=', d)) => match d {
                Some('=') => Token::EqualEqual,
                _ => Token::Equal,
            },
            Some(('*', d)) => match d {
                Some('=') => Token::MulEqual,
                _ => Token::Star,
            },
            Some(('/', d)) => match d {
                Some('=') => Token::DivEqual,
                _ => Token::Slash,
            },
            Some(('+', d)) => match d {
                Some('=') => Token::PlusEqual,
                _ => Token::Plus,
            },
            Some(('-', d)) => match d {
                Some('=') => Token::MinusEqual,
                _ => Token::Minus,
            },
            Some(('!', d)) => match d {
                Some('=') => Token::NotEqual,
                _ => Token::Not,
            },
            Some(('.', d)) => match d {
                Some('.') => Token::Range,
                _ => Token::Dot,
            },
            Some((c, _)) if c.is_digit(10) => {
                let string = self.read_number(c);
                if string.contains(".") {
                    Token::Float(string)
                } else {
                    Token::Int(string)
                }
            }
            Some((c, _)) if c.is_ascii_lowercase() => {
                Token::Identifier(self.read_identifier(c))
            }
            Some(('@', _)) => Token::True,

            // XXX: Temporary
            Some((c, _)) => Token::Unknown(c),

            None => Token::Eof,
        };

        TokenWithPosition {
            token,
            line_no,
            col_no,
            length: self.col_no - col_no,
        }
    }

    /// Consume and return the next char in the stream.
    fn next(&mut self) -> Option<(char, Option<char>)> {
        match self.stream.next() {
            Some(c) => {
                self.update_line_and_col_no(c);
                Some((c, self.lookahead_stream.next()))
            }
            _ => None,
        }
    }

    /// Consume and return the next char and next lookahead char if the
    /// next char matches the specified condition.
    fn next_if(
        &mut self,
        func: impl FnOnce(&char) -> bool,
    ) -> Option<(char, Option<char>)> {
        match self.stream.next_if(func) {
            Some(c) => {
                self.update_line_and_col_no(c);
                Some((c, self.lookahead_stream.next()))
            }
            _ => None,
        }
    }

    /// Consume and return the next char and next lookahead char if
    /// *both* the next char and next lookahead char match their
    /// respective conditions.
    fn next_if_both(
        &mut self,
        c_func: impl FnOnce(&char) -> bool,
        d_func: impl FnOnce(&char) -> bool,
    ) -> Option<(char, Option<char>)> {
        match (self.stream.peek(), self.lookahead_stream.peek()) {
            (Some(c), Some(d)) => {
                match c_func(c) && d_func(d) {
                    true => self.next(),
                    false => None,
                }
            }
            _ => None,
        }
    }

    /// Update line and column numbers *every* time a character is
    /// consumed from the stream.
    fn update_line_and_col_no(&mut self, c: char) {
        if c == '\n' {
            self.line_no += 1;
            self.col_no = 1;
        } else {
            self.col_no += 1;
        }
    }

    /// Look at the next character but don't consume it.
    fn peek(&mut self) -> Option<(&char, Option<&char>)> {
        match self.stream.peek() {
            Some(c) => Some((c, self.lookahead_stream.peek())),
            None => None,
        }
    }

    /// Consume and discard contiguous whitespace until a non-whitespace
    /// character or EOF is reached.
    fn skip_whitespace(&mut self) {
        loop {
            match self.next_if(|&c| c.is_whitespace()) {
                Some(_) => (),
                None => break,
            }
        }
    }

    /// Read contiguous digits and an optional decimal point into a new
    /// string. If a dot is encountered, it will be included only if the
    /// char following the dot is not another dot (because two dots are
    /// used for the range operator).
    fn read_number(&mut self, first_digit: char) -> String {
        let mut string = String::new();
        string.push(first_digit);
        loop {
            match self.next_if(|&c| c.is_digit(10)) {
                Some((digit, _)) => string.push(digit),
                None => break,
            }
        }
        match self.next_if_both(|&c| c == '.', |&d| d != '.') {
            // Number is followed by a dot and some other char; consume
            // the dot and any following digits.
            Some((dot, _)) => {
                string.push(dot);
                loop {
                    match self.next_if(|&c| c.is_digit(10)) {
                        Some((digit, _)) => string.push(digit),
                        None => break,
                    }
                }
            }
            // Number is followed by two dots, which is the range
            // operator.
            _ => (),
        }
        string
    }

    /// Read characters inside quotes into a new string. Note that the
    /// returned string does *not* include the opening and closing quote
    /// characters. Quotes can be embedded in a string by backslash-
    /// escaping them.
    fn read_string(&mut self, quote: char) -> String {
        let mut string = String::new();
        loop {
            match self.next() {
                Some(('\\', Some(d))) if d == quote => {
                    string.push(d);
                    self.next();
                }
                Some((c, _)) if c == quote => break,
                Some((c, _)) => string.push(c),
                None => break,
            }
        }
        string
    }

    /// Read starting from comment character to the end of the line.
    /// Note that the opening comment character is *not* included in the
    /// returned comment string. Leading and trailing whitespace is also
    /// stripped.
    fn read_comment(&mut self) -> String {
        let mut string = String::new();
        loop {
            match self.next_if(|&c| c != '\n') {
                Some((c, _)) => string.push(c),
                None => break,
            }
        }
        string.trim().to_string()
    }

    /// Read variable/function identifier.
    ///
    /// - Start with a lower case ASCII letter (a-z)
    /// - Contain only lower case ASCII letters, numbers, and underscores
    /// - End with a lower case ASCII letter or number
    ///
    /// NOTE: Identifiers that don't end with a char as noted above will
    ///       cause an error later.
    fn read_identifier(&mut self, first_char: char) -> String {
        let mut string = String::new();
        string.push(first_char);
        loop {
            match self.next_if(
                |&c| c.is_ascii_lowercase() || c.is_digit(10) || c == '_'
            ) {
                Some((c, _)) => string.push(c),
                None => break,
            }
        }
        string
    }
}
