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
}

#[derive(Debug, PartialEq)]
pub struct TokenWithPosition {
    pub token: Token,
    pub line_no: usize,
    pub col_no: usize,
    // The total length of a token, including quotes, newlines,
    // backslashes, etc.
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
        }
    }

    /// Scan text and return tokens. If an error is encountered, an
    /// error result will be returned containing a special token that
    /// indicates what the problem is along with all the tokens that
    /// were successfully parsed prior to the error.
    ///
    /// The possible error conditions are:
    ///
    /// - Unknown token
    /// - More input needed (e.g., unclosed string or group)
    ///
    /// In the first case, that's a syntax error.
    ///
    /// In the latter case, when more input is needed, you have to do
    /// something like this in the calling code, which is pretty clunky:
    ///
    /// ```
    /// // In this example, the string doesn't have a closing quote so
    /// // more input is needed to complete the scan.
    /// let scanner = Scanner::new();
    /// let source = "s = \"abc";
    /// let tokens = match scanner.scan(source) {
    ///     Ok(tokens) => tokens,
    ///     Err((error_token, tokens)) => match error_token.token {
    ///         Token::NeedsMoreInput(remaining_input) => {
    ///             let input = format!("{}\"", remaining_input);
    ///             match scanner.scan(input.as_str()) {
    ///                 Ok(tokens) => tokens,
    ///                 Err((error_token, tokens)) => {
    ///                     // Oops
    ///                 },
    ///             }
    ///         }
    ///     }
    /// };
    /// ```
    ///
    /// TODO: Find a better way to handle this^. Figure out how to store
    ///       the remaining, unparsed source internally or find some
    ///       other way such that the caller doesn't need to deal with
    ///       this in such a manual way.
    pub fn scan(
        &mut self,
        source: &'a str,
    ) -> Result<Vec<TokenWithPosition>, (TokenWithPosition, Vec<TokenWithPosition>)> {
        let mut tokens: Vec<TokenWithPosition> = vec![];

        self.stream = source.chars().peekable();
        self.lookahead_stream = source.chars().peekable();
        self.lookahead_stream.next();

        loop {
            let token_with_position = self.next_token();
            match token_with_position.token {
                Token::Unknown(_) => {
                    return Err((token_with_position, tokens));
                }
                Token::NeedsMoreInput(_) => {
                    return Err((token_with_position, tokens));
                }
                Token::EndOfInput => {
                    tokens.push(token_with_position);
                    break;
                }
                _ => tokens.push(token_with_position),
            }
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> TokenWithPosition {
        let line_no = self.line_no;
        let col_no = self.col_no;
        let mut token_length: Option<usize> = None;

        let token = match self.next() {
            Some(('"', _)) => match self.read_string('"') {
                (string, length, true) => {
                    token_length = Some(length);
                    Token::String(string)
                }
                (string, length, false) => {
                    if length > self.col_no {
                        self.col_no = 1;
                    } else {
                        self.col_no -= length;
                    }
                    token_length = Some(length);
                    Token::NeedsMoreInput(format!("\"{}", string))
                }
            },
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
            Some(('-', Some('>'))) => self.next_and_token(Token::BlockStart),
            Some(('-', _)) => Token::Minus,
            Some(('!', Some('='))) => self.next_and_token(Token::NotEqual),
            Some(('!', Some('!'))) => self.next_and_token(Token::AsBool),
            Some(('!', _)) => Token::Not,
            Some(('.', Some('.'))) => self.next_and_token(Token::Range),
            Some(('.', _)) => Token::Dot,
            Some(('%', _)) => Token::Percent,
            Some(('^', _)) => Token::Caret,
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
            Some((c, _)) if c.is_whitespace() => Token::Whitespace(self.read_whitespace(c)),
            Some((c, _)) => Token::Unknown(c),
            None => Token::EndOfInput,
        };

        // In most cases, the length of a token can be calculated
        // automatically from the current column position and the
        // previous position. The exception is for tokens that can span
        // multiple lines, such as strings.
        let length = match token_length {
            Some(length) => length,
            None => self.col_no - col_no,
        };

        TokenWithPosition::new(token, line_no, col_no, length)
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

    /// Read contiguous whitespace.
    fn read_whitespace(&mut self, first_char: char) -> String {
        let mut string = String::new();
        string.push(first_char);
        loop {
            match self.next_if(|&c| c.is_whitespace()) {
                Some((c, _)) => string.push(c),
                None => break,
            }
        }
        string
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
    fn read_string(&mut self, quote: char) -> (String, usize, bool) {
        let mut string = String::new();
        let mut length = 1;
        loop {
            match self.next() {
                // Skip newlines preceded by backslash
                Some(('\\', Some('\n'))) => {
                    self.next();
                    length += 1;
                }
                // Handle embedded/escaped quotes
                Some(('\\', Some(d))) if d == quote => {
                    self.next();
                    string.push(d);
                    length += 1;
                }
                // Found closing quote; return string
                Some((c, _)) if c == quote => return (string, length + 1, true),
                // Append current char and continue
                Some((c, _)) => string.push(c),
                // End of input reached without finding closing quote :(
                None => return (string, length, false),
            }
            length += 1;
        }
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
