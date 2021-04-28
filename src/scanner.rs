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
            Some((',', _)) => Token::Comma,
            Some(('(', _)) => Token::LeftParen,
            Some((')', _)) => Token::RightParen,
            Some(('[', _)) => Token::LeftSquareBracket,
            Some((']', _)) => Token::RightSquareBracket,
            Some(('<', Some('='))) => self.next_and_token(Token::LessThanOrEqual),
            Some(('<', Some('-'))) => self.next_and_token(Token::LoopFeed),
            Some(('<', _)) => Token::LeftAngleBracket,
            Some(('>', Some('='))) => self.next_and_token(Token::GreaterThanOrEqual),
            Some(('>', _)) => Token::RightAngleBracket,
            Some(('=', Some('='))) => self.next_and_token(Token::EqualEqual),
            Some(('=', _)) => Token::Equal,
            Some(('&', Some('&'))) => self.next_and_token(Token::And),
            Some(('|', Some('|'))) => self.next_and_token(Token::Or),
            Some(('*', Some('='))) => self.next_and_token(Token::MulEqual),
            Some(('*', _)) => Token::Star,
            Some(('/', Some('='))) => self.next_and_token(Token::DivEqual),
            Some(('/', _)) => Token::Slash,
            Some(('+', Some('='))) => self.next_and_token(Token::PlusEqual),
            Some(('+', _)) => Token::Plus,
            Some(('-', Some('='))) => self.next_and_token(Token::MinusEqual),
            Some(('-', Some('>'))) => self.next_and_token(Token::ReturnType),
            Some(('-', _)) => Token::Minus,
            Some(('!', Some('='))) => self.next_and_token(Token::NotEqual),
            Some(('!', Some('!'))) => self.next_and_token(Token::AsBool),
            Some(('!', _)) => Token::Not,
            Some(('.', Some('.'))) => self.next_and_token(Token::Range),
            Some(('.', _)) => Token::Dot,
            Some((c @ '0'..='9', _)) => match self.read_number(c) {
                string if string.contains(".") => Token::Float(string),
                string => Token::Int(string),
            },
            Some((c @ 'a'..='z', _)) => Token::Identifier(self.read_identifier(c)),
            Some((c @ 'A'..='Z', _)) => Token::TypeIdentifier(self.read_type_identifier(c)),
            Some((c @ '@', Some('a'..='z'))) => {
                Token::TypeMethodIdentifier(self.read_identifier(c))
            }
            Some((c @ '$', Some('a'..='z'))) => {
                Token::SpecialMethodIdentifier(self.read_identifier(c))
            }
            Some((c, _)) => Token::Unknown(c),  // XXX: Temporary
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

    /// Consume the next char in the stream and return the specified
    /// token.
    fn next_and_token(&mut self, token: Token) -> Token {
        self.next();
        token
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
    /// char following the dot is another digit.
    fn read_number(&mut self, first_digit: char) -> String {
        let mut string = String::new();
        string.push(first_digit);
        loop {
            match self.next_if(|&c| c.is_digit(10)) {
                Some((digit, _)) => string.push(digit),
                None => break,
            }
        }
        match self.next_if_both(|&c| c == '.', |&d| d.is_digit(10)) {
            // If the number is followed by a dot and at least one
            // digit consume the dot, the digit, and any following
            // digits.
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
    /// Identifiers:
    ///
    /// - start with a lower case ASCII letter (a-z)
    /// - contain lower case ASCII letters, numbers, and underscores
    /// - end with a lower case ASCII letter or number
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

    /// Read type identifier.
    ///
    /// Type identifiers:
    ///
    /// - start with an upper case ASCII letter (A-Z)
    /// - contain ASCII letters and numbers
    fn read_type_identifier(&mut self, first_char: char) -> String {
        let mut string = String::new();
        string.push(first_char);
        loop {
            match self.next_if(|&c| c.is_ascii_alphabetic() || c.is_digit(10)) {
                Some((c, _)) => string.push(c),
                None => break,
            }
        }
        string
    }
}
