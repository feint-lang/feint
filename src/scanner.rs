use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

use crate::tokens::Token;
use std::process::id;

pub struct Scanner<'a> {
    stream: Peekable<Chars<'a>>,
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
        Scanner {
            stream: source.chars().peekable(),
            line_no: 1,
            col_no: 1,
        }
    }

    pub fn scan(&mut self) -> Vec<TokenWithPosition> {
        let mut tokens: Vec<TokenWithPosition> = vec!();
        loop {
            self.skip_whitespace();
            let token = self.next_token();
            if token.token == Token::Eof {
                tokens.push(token);
                break;
            } else {
                tokens.push(token);
            }
        }
        tokens
    }

    fn next_token(&mut self) -> TokenWithPosition {
        let line_no = self.line_no;
        let col_no = self.col_no;

        let token = match self.next_char() {
            Some(c @ '"') => Token::String(self.read_string(c)),
            Some(c @ '#') => Token::Comment(self.read_comment(c)),
            Some('(') => Token::LeftParen,
            Some(')') => Token::RightParen,
            Some('[') => Token::LeftSquareBracket,
            Some(']') => Token::RightSquareBracket,
            Some('<') => Token::LeftAngleBracket,
            Some('>') => Token::RightAngleBracket,
            Some('=') => Token::Equal,
            Some('*') => Token::Star,
            Some('/') => Token::Slash,
            Some('+') => Token::Plus,
            Some('-') => Token::Minus,
            Some('!') => self.token_if_next_char('=', Token::NotEqual, Token::Not),
            Some('.') => {
                match self.next_char_if(|&c| c == '.') {
                    Some(_) => Token::Range,
                    None => Token::Dot,
                }
            }
            Some(c) if c.is_digit(10) => {
                let string = self.read_number(c);
                if string.contains(".") {
                    Token::Float(string)
                } else {
                    Token::Int(string)
                }
            }
            Some(c) if c.is_ascii_lowercase() => {
                Token::Identifier(self.read_identifier(c))
            }
            Some('@') => Token::True,

            // XXX: Temporary
            Some(c) => Token::Unknown(c),

            None => Token::Eof,
        };

        TokenWithPosition {
            token,
            line_no,
            col_no,
            length: self.col_no - col_no,
        }
    }

    /// Return token if next char is the specified char. Otherwise,
    /// return the default token.
    fn token_if_next_char(&mut self, c: char, token: Token, default: Token) -> Token {
        match self.next_char_if(|&next| next == c) {
            Some(_) => token,
            None => default,
        }
    }

    /// Consume and return the next char in the stream.
    fn next_char(&mut self) -> Option<char> {
        match self.stream.next() {
            Some(c) => {
                self.update_line_and_col_no(c);
                Some(c)
            }
            _ => None,
        }
    }

    /// Consume and return the next char in the stream *if* it matches
    /// the specified condition.
    fn next_char_if(&mut self, func: impl FnOnce(&char) -> bool) -> Option<char> {
        match self.stream.next_if(func) {
            Some(c) => {
                self.update_line_and_col_no(c);
                Some(c)
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
    fn peek_char(&mut self) -> Option<&char> {
        self.stream.peek()
    }

    /// Consume and discard contiguous whitespace until a non-whitespace
    /// character or EOF is reached.
    fn skip_whitespace(&mut self) {
        loop {
            match self.next_char_if(|&c| c.is_whitespace()) {
                Some(_) => (),
                None => break,
            }
        }
    }

    /// Read contiguous digits into a new string.
    fn read_number(&mut self, first_digit: char) -> String {
        let mut string = String::new();
        string.push(first_digit);
        loop {
            match self.next_char_if(|&c| c.is_digit(10)) {
                Some(c) => string.push(c),
                None => break,
            }
        }
        string
    }

    /// Read characters inside quotes into a new string. Note that the
    /// returned string does *not* include the opening and closing quote
    /// characters. Quotes can be embedded in a string by backslash-
    /// escaping them.
    fn read_string(&mut self, quote: char) -> String {
        let mut string = String::new();
        let mut previous_char: char = quote;
        loop {
            match self.next_char() {
                Some(c) if c == quote && previous_char == '\\' => {
                    string.pop();
                    string.push(c);
                    previous_char = c;
                }
                Some(c) if c == quote => {
                    break;
                }
                Some(c) => {
                    string.push(c);
                    previous_char = c;
                }
                None => break,
            }
        }
        string
    }

    /// Read starting from comment character to the end of the line.
    /// Note that the opening comment character is *not* included in the
    /// returned comment string. Leading and trailing whitespace is also
    /// stripped.
    fn read_comment(&mut self, comment_char: char) -> String {
        let mut string = String::new();
        string.push(comment_char);
        loop {
            match self.next_char_if(|&c| c != '\n') {
                Some(c) => string.push(c),
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
            match self.next_char_if(
                |&c| c.is_ascii_lowercase() || c.is_digit(10) || c == '_'
            ) {
                Some(c) => string.push(c),
                None => break,
            }
        }
        string
    }
}
