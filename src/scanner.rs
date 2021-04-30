use std::fmt;
use std::iter::{Chain, Peekable};
use std::str::Chars;

use crate::tokens::Token;

pub struct Scanner<'a> {
    /// Stream of input characters from input string
    stream: Peekable<Chars<'a>>,
    /// The same stream but one character ahead for easier lookaheads
    lookahead_stream: Peekable<Chars<'a>>,
    line_no: usize,
    col_no: usize,
    remaining_source: String,
}

#[derive(Debug, PartialEq)]
pub struct TokenWithPosition {
    pub token: Token,
    pub line_no: usize,
    pub col_no: usize,
    pub length: usize,
}

impl TokenWithPosition {
    pub fn new(token: Token, line_no: usize, col_no: usize, length: usize) -> TokenWithPosition {
        TokenWithPosition {
            token,
            line_no,
            col_no,
            length,
        }
    }
}

impl fmt::Display for TokenWithPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Token {}:{}:{} {:?}",
            self.line_no, self.col_no, self.length, self.token,
        )
    }
}

impl<'a> Scanner<'a> {
    pub fn new() -> Scanner<'a> {
        Scanner {
            stream: "".chars().peekable(),
            lookahead_stream: "".chars().peekable(),
            line_no: 1,
            col_no: 1,
            remaining_source: "".to_string(),
        }
    }

    pub fn scan(
        &mut self,
        source: String,
        finalize: bool,
    ) -> Result<Vec<TokenWithPosition>, String> {
        let mut tokens: Vec<TokenWithPosition> = vec![];

        // if self.remaining_source.len() > 0 {
        //     self.scan(self.remaining_source.clone(), false);
        // }

        self.stream = source.chars().peekable();
        self.lookahead_stream = source.chars().peekable();
        self.lookahead_stream.next();

        loop {
            let token_with_position = self.next_token();
            let length = token_with_position.length;
            match token_with_position.token {
                Token::EndOfInput => {
                    tokens.push(token_with_position);
                    break;
                }
                Token::Unknown(c) => {
                    return Err(format!("Encountered unknown token: {}", c));
                }
                // The length of a string should be 2 less than the
                // length of its token.
                Token::String(string) if length - string.len() == 1 => {
                    self.col_no -= string.len() + 1;
                    tokens.push(TokenWithPosition {
                        token: Token::NeedsMoreInput(format!("\"{}", string)),
                        line_no: token_with_position.line_no,
                        col_no: token_with_position.col_no,
                        length: token_with_position.length,
                    });
                    break;
                }
                _ => {
                    tokens.push(token_with_position);
                }
            }
            self.skip_whitespace();
        }

        if finalize {
            match tokens.last() {
                Some(TokenWithPosition {
                    token: Token::NeedsMoreInput(_),
                    line_no: _,
                    col_no: _,
                    length: _,
                }) => return Err("More input needed".to_string()),
                Some(_) => (),
                None => (),
            }
        }

        Ok(tokens)
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
            Some((c, _)) => Token::Unknown(c),
            None => Token::EndOfInput,
        };

        TokenWithPosition::new(token, line_no, col_no, self.col_no - col_no)
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
    fn next_if(&mut self, func: impl FnOnce(&char) -> bool) -> Option<(char, Option<char>)> {
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
            (Some(c), Some(d)) => match c_func(c) && d_func(d) {
                true => self.next(),
                false => None,
            },
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
            match self.next_if(|&c| c.is_ascii_lowercase() || c.is_digit(10) || c == '_') {
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
