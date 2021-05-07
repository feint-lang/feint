use std::iter::Peekable;
use std::str::Chars;

use crate::stack::Stack;
use crate::tokens::{Token, TokenWithPosition};

type NextOption = Option<(char, Option<char>, Option<char>)>;
type PeekOption<'a> = Option<(&'a char, Option<&'a char>, Option<&'a char>)>;

/// Create a scanner with the specified source, scan the source, and
/// return the resulting tokens or error.
pub fn scan(
    source: &str,
    line_no: usize,
    col_no: usize,
) -> Result<Vec<TokenWithPosition>, (TokenWithPosition, Vec<TokenWithPosition>)> {
    let mut scanner = Scanner::new(source, line_no, col_no);
    scanner.scan()
}

pub struct Scanner<'a> {
    source: &'a str,
    /// Stream of input characters from input string
    stream: Peekable<Chars<'a>>,
    /// The same stream but one character ahead for easier lookaheads
    one_ahead_stream: Peekable<Chars<'a>>,
    /// The same stream but two characters ahead for easier lookaheads
    two_ahead_stream: Peekable<Chars<'a>>,
    line_no: usize,
    col_no: usize,
    bracket_stack: Stack<TokenWithPosition>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str, line_no: usize, col_no: usize) -> Self {
        let stream = source.chars().peekable();
        let mut one_ahead_stream = source.chars().peekable();
        let mut two_ahead_stream = source.chars().peekable();
        one_ahead_stream.next();
        two_ahead_stream.next();
        two_ahead_stream.next();
        Scanner {
            source,
            stream,
            one_ahead_stream,
            two_ahead_stream,
            line_no,
            col_no,
            bracket_stack: Stack::new(),
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
    /// - Unexpected whitespace after indent
    /// - Unterminated string
    ///
    /// The first two cases are syntax errors.
    ///
    /// In the case of an unterminated string, the original string plus
    /// additional input can be re-scanned.
    ///
    /// TODO: Find a better way to handle this^. Figure out how to store
    ///       the remaining, unparsed source internally or find some
    ///       other way such that the caller doesn't need to deal with
    ///       this in such a clunky way.
    pub fn scan(
        &mut self,
    ) -> Result<Vec<TokenWithPosition>, (TokenWithPosition, Vec<TokenWithPosition>)> {
        let mut tokens: Vec<TokenWithPosition> = vec![];
        let mut previous_was_indent_0 = true;
        loop {
            let token_with_position = self.next_token();
            match token_with_position.token {
                Token::Unknown(_) | Token::UnterminatedString(_) | Token::UnexpectedWhitespace => {
                    break Err((token_with_position, tokens))
                }
                Token::Indent(0) => {
                    // In effect, collapse contiguous blank lines
                    if !previous_was_indent_0 {
                        tokens.push(token_with_position);
                    }
                    previous_was_indent_0 = true;
                }
                Token::EndOfInput => {
                    // End-of-input is handled like a newline.
                    if !previous_was_indent_0 {
                        tokens.push(TokenWithPosition::new(
                            Token::Indent(0),
                            token_with_position.line_no,
                            token_with_position.col_no,
                        ));
                    }
                    break match self.bracket_stack.peek() {
                        Some(t) => Err((t.clone(), tokens)),
                        None => Ok(tokens),
                    };
                }
                Token::LeftParen | Token::LeftSquareBracket => {
                    self.bracket_stack.push(token_with_position.clone());
                    tokens.push(token_with_position);
                    previous_was_indent_0 = false;
                    self.consume_whitespace();
                }
                Token::RightParen | Token::RightSquareBracket => {
                    match self.bracket_stack.pop() {
                        Some(_) => (),
                        _ => break Err((token_with_position, tokens)),
                    }
                    tokens.push(token_with_position);
                    previous_was_indent_0 = false;
                    self.consume_whitespace();
                }
                _ => {
                    tokens.push(token_with_position);
                    previous_was_indent_0 = false;
                    self.consume_whitespace();
                }
            }
        }
    }

    fn next_token(&mut self) -> TokenWithPosition {
        let mut start_line_no = self.line_no;
        let mut start_col_no = self.col_no;

        let token = match self.next() {
            Some(('"', _, _)) => match self.read_string('"') {
                // Terminated
                (string, true) => Token::String(string),
                // Unterminated
                (string, false) => {
                    self.line_no = start_line_no;
                    self.col_no = start_col_no;
                    Token::UnterminatedString(format!("\"{}", string))
                }
            },
            Some(('#', _, _)) => Token::Comment(self.read_comment()),
            Some((':', _, _)) => Token::Colon,
            Some((',', _, _)) => Token::Comma,
            Some(('(', _, _)) => Token::LeftParen,
            Some((')', _, _)) => Token::RightParen,
            Some(('[', _, _)) => Token::LeftSquareBracket,
            Some((']', _, _)) => Token::RightSquareBracket,
            Some(('<', Some('='), _)) => self.next_and_token(Token::LessThanOrEqual),
            Some(('<', Some('-'), _)) => self.next_and_token(Token::LoopFeed),
            Some(('<', _, _)) => Token::LeftAngleBracket,
            Some(('>', Some('='), _)) => self.next_and_token(Token::GreaterThanOrEqual),
            Some(('>', _, _)) => Token::RightAngleBracket,
            Some(('=', Some('='), _)) => self.next_and_token(Token::EqualEqual),
            Some(('=', _, _)) => Token::Equal,
            Some(('&', Some('&'), _)) => self.next_and_token(Token::And),
            Some(('&', _, _)) => self.next_and_token(Token::Ampersand),
            Some(('|', Some('|'), _)) => self.next_and_token(Token::Or),
            Some(('|', _, _)) => self.next_and_token(Token::Pipe),
            Some(('*', Some('*'), _)) => self.next_and_token(Token::DoubleStar),
            Some(('*', Some('='), _)) => self.next_and_token(Token::MulEqual),
            Some(('*', _, _)) => Token::Star,
            Some(('/', Some('='), _)) => self.next_and_token(Token::DivEqual),
            Some(('/', _, _)) => Token::Slash,
            Some(('+', Some('='), _)) => self.next_and_token(Token::PlusEqual),
            Some(('+', _, _)) => Token::Plus,
            Some(('-', Some('='), _)) => self.next_and_token(Token::MinusEqual),
            Some(('-', Some('>'), _)) => self.next_and_token(Token::FuncStart),
            Some(('-', _, _)) => Token::Minus,
            Some(('!', Some('='), _)) => self.next_and_token(Token::NotEqual),
            Some(('!', Some('!'), _)) => self.next_and_token(Token::AsBool),
            Some(('!', _, _)) => Token::Not,
            Some(('.', Some('.'), Some('.'))) => self.next_and_token(Token::RangeInclusive),
            Some(('.', Some('.'), _)) => self.next_and_token(Token::Range),
            Some(('.', _, _)) => Token::Dot,
            Some(('%', _, _)) => Token::Percent,
            Some(('^', _, _)) => Token::Caret,
            Some((c @ '0'..='9', _, _)) => match self.read_number(c) {
                string if string.contains(".") || string.contains("E") => Token::Float(string),
                string => Token::Int(string),
            },
            Some((c @ 'a'..='z', _, _)) => {
                let identifier = self.read_identifier(c);
                if identifier == "true" {
                    Token::True
                } else if identifier == "false" {
                    Token::False
                } else {
                    Token::Identifier(identifier)
                }
            }
            Some((c @ 'A'..='Z', _, _)) => Token::TypeIdentifier(self.read_type_identifier(c)),
            Some((c @ '@', Some('a'..='z'), _)) => {
                Token::TypeMethodIdentifier(self.read_identifier(c))
            }
            Some((c @ '$', Some('a'..='z'), _)) => {
                Token::SpecialMethodIdentifier(self.read_identifier(c))
            }

            Some(('\n', _, _)) => {
                start_line_no = self.line_no;
                start_col_no = self.col_no;

                let indent_level = self.read_indent();
                let whitespace_count = self.consume_whitespace();

                match self.stream.peek() {
                    // Blank or whitespace-only line
                    Some('\n') | None => Token::Indent(0),
                    Some(_) => match whitespace_count {
                        // Indent followed by non-whitespace character
                        0 => Token::Indent(indent_level),
                        // Indent followed by unexpected whitespace
                        _ => Token::UnexpectedWhitespace,
                    },
                }
            }

            Some((c, _, _)) => Token::Unknown(c),

            None => Token::EndOfInput,
        };

        TokenWithPosition::new(token, start_line_no, start_col_no)
    }

    /// Consume and return the next char in the stream.
    fn next(&mut self) -> NextOption {
        match self.stream.next() {
            Some(c) => {
                self.update_line_and_col_no(c);
                Some((
                    c,
                    self.one_ahead_stream.next(),
                    self.two_ahead_stream.next(),
                ))
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
    fn next_if(&mut self, func: impl FnOnce(&char) -> bool) -> NextOption {
        match self.stream.next_if(func) {
            Some(c) => {
                self.update_line_and_col_no(c);
                Some((
                    c,
                    self.one_ahead_stream.next(),
                    self.two_ahead_stream.next(),
                ))
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
    ) -> NextOption {
        match (self.stream.peek(), self.one_ahead_stream.peek()) {
            (Some(c), Some(d)) => match c_func(c) && d_func(d) {
                true => self.next(),
                false => None,
            },
            _ => None,
        }
    }

    fn next_if_all(
        &mut self,
        c_func: impl FnOnce(&char) -> bool,
        d_func: impl FnOnce(&char) -> bool,
        e_func: impl FnOnce(&char) -> bool,
    ) -> NextOption {
        match (
            self.stream.peek(),
            self.one_ahead_stream.peek(),
            self.two_ahead_stream.peek(),
        ) {
            (Some(c), Some(d), Some(e)) => match c_func(c) && d_func(d) && e_func(e) {
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

    /// Consume contiguous whitespace up to the end of the line. Return
    /// the number of whitespace characters consumed.
    fn consume_whitespace(&mut self) -> usize {
        let mut count = 0;
        loop {
            match self.next_if(|&c| c.is_whitespace() && c != '\n') {
                Some(_) => count += 1,
                None => break count,
            }
        }
    }

    /// Returns the number of contiguous space characters at the start
    /// of a line. An indent is defined as N space characters followed
    /// by a non-whitespace character.
    fn read_indent(&mut self) -> u8 {
        let mut count = 0;
        loop {
            match self.next_if(|&c| c == ' ') {
                Some(_) => count += 1,
                None => break count,
            }
        }
    }

    /// Read contiguous digits and an optional decimal point into a new
    /// string. If a dot is encountered, it will be included only if the
    /// char following the dot is another digit.
    fn read_number(&mut self, first_digit: char) -> String {
        let mut string = first_digit.to_string();
        string.push_str(self.collect_digits().as_str());
        match self.next_if_both(|&c| c == '.', |&d| d.is_digit(10)) {
            // If the number is followed by a dot and at least one
            // digit consume the dot, the digit, and any following
            // digits.
            Some((dot, _, _)) => {
                string.push(dot);
                string.push_str(self.collect_digits().as_str());
            }
            _ => (),
        }
        // The next two match blocks handle E notation. The first
        // matches WITHOUT a sign before the E and the second matches
        // WITH a sign before the E. Lower or uppercase E is accepted
        // and will be normalized to uppercase.
        // TODO: Make this less verbose?
        match self.next_if_both(|&c| c == 'e' || c == 'E', |&e| e.is_digit(10)) {
            Some((_e, _digit, _)) => {
                string.push('E');
                string.push_str(self.collect_digits().as_str());
            }
            _ => (),
        }
        match self.next_if_all(
            |&c| c == 'e' || c == 'E',
            |&d| d == '+' || d == '-',
            |&e| e.is_digit(10),
        ) {
            Some((_e, Some(sign), _digit)) => {
                self.next(); // Skip over sign
                string.push('E');
                string.push(sign);
                string.push_str(self.collect_digits().as_str());
            }
            _ => (),
        }
        string
    }

    fn collect_digits(&mut self) -> String {
        let mut digits = String::new();
        loop {
            match self.next_if(|&c| c.is_digit(10)) {
                Some((digit, _, _)) => digits.push(digit),
                None => break digits,
            }
        }
    }

    /// Read characters inside quotes into a new string. Note that the
    /// returned string does *not* include the opening and closing quote
    /// characters. Quotes can be embedded in a string by backslash-
    /// escaping them.
    fn read_string(&mut self, quote: char) -> (String, bool) {
        let mut string = String::new();
        loop {
            match self.next() {
                // Skip newlines preceded by backslash
                Some(('\\', Some('\n'), _)) => {
                    self.next();
                }
                // Handle embedded/escaped quotes
                Some(('\\', Some(d), _)) if d == quote => {
                    self.next();
                    string.push(d);
                }
                // Found closing quote; return string
                Some((c, _, _)) if c == quote => break (string, true),
                // Append current char and continue
                Some((c, _, _)) => string.push(c),
                // End of input reached without finding closing quote :(
                None => break (string, false),
            }
        }
    }

    /// Read starting from comment character to the end of the line.
    /// Note that the opening comment character is *not* included in the
    /// returned comment string. Leading and trailing whitespace is also
    /// stripped.
    fn read_comment(&mut self) -> String {
        let mut comment = String::new();
        loop {
            match self.next_if(|&c| c != '\n') {
                Some((c, _, _)) => comment.push(c),
                None => break comment.trim().to_string(),
            }
        }
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
        let mut string = first_char.to_string();
        loop {
            match self.next_if(|&c| c.is_ascii_lowercase() || c.is_digit(10) || c == '_') {
                Some((c, _, _)) => string.push(c),
                None => break string,
            }
        }
    }

    /// Read type identifier.
    ///
    /// Type identifiers:
    ///
    /// - start with an upper case ASCII letter (A-Z)
    /// - contain ASCII letters and numbers
    fn read_type_identifier(&mut self, first_char: char) -> String {
        let mut string = first_char.to_string();
        loop {
            match self.next_if(|&c| c.is_ascii_alphabetic() || c.is_digit(10)) {
                Some((c, _, _)) => string.push(c),
                None => break string,
            }
        }
    }
}
