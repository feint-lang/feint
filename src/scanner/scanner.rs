use std::iter::Peekable;
use std::str::Chars;

use crate::util::Stack;

use super::{Location, ScanError, ScanErrorType, ScanResult, Token, TokenWithLocation};

type NextOption = Option<(char, Option<char>, Option<char>)>;
type NextTwoOption = Option<(char, char, Option<char>)>;
type NextThreeOption = Option<(char, char, char)>;
type PeekOption<'a> = Option<(&'a char, Option<&'a char>, Option<&'a char>)>;

/// Create a scanner with the specified text source, scan the text, and
/// return the resulting tokens or error.
pub fn scan(text: &str) -> Result<Vec<TokenWithLocation>, ScanError> {
    let mut scanner = Scanner::new(text, Location::new(1, 1));
    let mut tokens: Vec<TokenWithLocation> = vec![];
    for item in scanner {
        match item {
            Ok(token) => {
                tokens.push(token);
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
    Ok(tokens)
}

pub struct Scanner<'a> {
    source: &'a str,
    /// Stream of input characters from input string
    stream: Peekable<Chars<'a>>,
    /// The same stream but one character ahead for easier lookaheads
    one_ahead_stream: Peekable<Chars<'a>>,
    /// The same stream but two characters ahead for easier lookaheads
    two_ahead_stream: Peekable<Chars<'a>>,
    location: Location,
    current_token: Option<Token>,
    indent_level: u8,
    bracket_stack: Stack<(char, Location)>,
}

impl<'a> Scanner<'a> {
    /// Create a new scanner from source text.
    /// XXX: Not sure if it's useful to be able to pass line & col no.
    pub fn new(source: &'a str, location: Location) -> Self {
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
            location,
            current_token: None,
            indent_level: 0,
            bracket_stack: Stack::new(),
        }
    }

    fn next_token(&mut self) -> ScanResult {
        let start = self.location;
        let mut end = Location::new(self.location.line, self.location.col - 1);

        let token = match self.next_char() {
            Some(('"', _, _)) => match self.read_string('"') {
                (string, true) => Token::String(string),
                (string, false) => {
                    return Err(ScanError::new(
                        ScanErrorType::UnterminatedString(format!("\"{}", string)),
                        start,
                    ));
                }
            },
            Some(('#', _, _)) => Token::Comment(self.read_comment()),
            Some((':', _, _)) => Token::Colon,
            Some((',', _, _)) => Token::Comma,
            Some((c @ '(', _, _)) => {
                self.bracket_stack.push((c, start));
                Token::LeftParen
            }
            Some((c @ ')', _, _)) => {
                match self.bracket_stack.pop() {
                    Some(('(', _)) => (),
                    None | Some(_) => {
                        return Err(ScanError::new(
                            ScanErrorType::UnmatchedClosingBracket(c),
                            start,
                        ));
                    }
                }
                Token::RightParen
            }
            Some((c @ '[', _, _)) => {
                self.bracket_stack.push((c, start));
                Token::LeftSquareBracket
            }
            Some((c @ ']', _, _)) => {
                match self.bracket_stack.pop() {
                    Some(('[', _)) => (),
                    None | Some(_) => {
                        return Err(ScanError::new(
                            ScanErrorType::UnmatchedClosingBracket(c),
                            start,
                        ));
                    }
                }
                Token::RightSquareBracket
            }
            Some(('<', Some('='), _)) => self.next_char_and_token(Token::LessThanOrEqual),
            Some(('<', Some('-'), _)) => self.next_char_and_token(Token::LoopFeed),
            Some((c @ '<', _, _)) => {
                self.bracket_stack.push((c, start));
                Token::LeftAngleBracket
            }
            Some(('>', Some('='), _)) => self.next_char_and_token(Token::GreaterThanOrEqual),
            Some((c @ '>', _, _)) => {
                match self.bracket_stack.pop() {
                    Some(('<', _)) => (),
                    None | Some(_) => {
                        return Err(ScanError::new(
                            ScanErrorType::UnmatchedClosingBracket(c),
                            start,
                        ));
                    }
                }
                Token::RightAngleBracket
            }
            Some(('=', Some('='), _)) => self.next_char_and_token(Token::EqualEqual),
            Some(('=', _, _)) => Token::Equal,
            Some(('&', Some('&'), _)) => self.next_char_and_token(Token::And),
            Some(('&', _, _)) => self.next_char_and_token(Token::Ampersand),
            Some(('|', Some('|'), _)) => self.next_char_and_token(Token::Or),
            Some(('|', _, _)) => self.next_char_and_token(Token::Pipe),
            Some(('*', Some('*'), _)) => self.next_char_and_token(Token::DoubleStar),
            Some(('*', Some('='), _)) => self.next_char_and_token(Token::MulEqual),
            Some(('*', _, _)) => Token::Star,
            Some(('/', Some('='), _)) => self.next_char_and_token(Token::DivEqual),
            Some(('/', _, _)) => Token::Slash,
            Some(('+', Some('='), _)) => self.next_char_and_token(Token::PlusEqual),
            Some(('+', _, _)) => Token::Plus,
            Some(('-', Some('='), _)) => self.next_char_and_token(Token::MinusEqual),
            Some(('-', Some('>'), _)) => self.next_char_and_token(Token::FuncStart),
            Some(('-', _, _)) => Token::Minus,
            Some(('!', Some('='), _)) => self.next_char_and_token(Token::NotEqual),
            Some(('!', Some('!'), _)) => self.next_char_and_token(Token::AsBool),
            Some(('!', _, _)) => Token::Not,
            Some(('.', Some('.'), Some('.'))) => {
                self.next_two_chars_and_token(Token::RangeInclusive)
            }
            Some(('.', Some('.'), _)) => self.next_char_and_token(Token::Range),
            Some(('.', _, _)) => Token::Dot,
            Some(('%', _, _)) => Token::Percent,
            Some(('^', _, _)) => Token::Caret,
            Some((c @ '0'..='9', _, _)) => match self.read_number(c) {
                (string, _) if string.contains(".") || string.contains("E") => Token::Float(string),
                (string, radix) => Token::Int(string, radix),
            },
            // Identifiers
            Some((c @ 'a'..='z', _, _)) => {
                let identifier = self.read_identifier(c);
                match identifier.as_str() {
                    "true" => Token::True,
                    "false" => Token::False,
                    _ => Token::Identifier(identifier),
                }
            }
            Some((c @ 'A'..='Z', _, _)) => Token::TypeIdentifier(self.read_type_identifier(c)),
            Some((c @ '@', Some('a'..='z'), _)) => {
                Token::TypeMethodIdentifier(self.read_identifier(c))
            }
            Some((c @ '$', Some('a'..='z'), _)) => {
                Token::SpecialMethodIdentifier(self.read_identifier(c))
            }
            // Newlines
            Some(('\n', _, _)) => {
                end = Location::new(end.line - 1, start.col);
                Token::Newline
            }
            Some(('\r', Some('\n'), _)) => {
                self.next_char();
                Token::Newline
            }
            // Indents
            Some((' ', _, _)) if self.current_token == Some(Token::Newline) => {
                let num_spaces = self.read_indent() + 1;
                let whitespace_count = self.consume_whitespace();
                match self.stream.peek() {
                    // Blank or whitespace-only line
                    Some('\n') | None => Token::Indent(0),
                    Some(_) => match whitespace_count {
                        0 => match num_spaces % 4 {
                            // Indent followed by non-whitespace character
                            0 => Token::Indent(num_spaces / 4),
                            // Indent followed by non-space whitespace character(s)
                            _ => {
                                return Err(ScanError::new(
                                    ScanErrorType::UnexpectedIndent(num_spaces),
                                    start,
                                ));
                            }
                        },
                        // Indent followed by unexpected whitespace
                        _ => {
                            return Err(ScanError::new(ScanErrorType::UnexpectedWhitespace, start));
                        }
                    },
                }
            }
            // Unknown
            Some((c, _, _)) => {
                return Err(ScanError::new(ScanErrorType::UnknownToken(c), start));
            }
            // End of input
            None => {
                let bracket = self.bracket_stack.pop();
                match bracket {
                    Some((c, location)) => {
                        return Err(ScanError::new(
                            ScanErrorType::UnmatchedOpeningBracket(c),
                            location,
                        ));
                    }
                    None => (),
                }
                Token::EndOfInput
            }
        };

        self.consume_whitespace();
        self.current_token = Some(token.clone());
        Ok(TokenWithLocation::new(token.clone(), start, end))
    }

    /// Consume and return the next character from each stream.
    fn next_char(&mut self) -> NextOption {
        match self.stream.next() {
            Some(c) => {
                self.update_location(c);
                Some((
                    c,
                    self.one_ahead_stream.next(),
                    self.two_ahead_stream.next(),
                ))
            }
            _ => None,
        }
    }

    fn peek_char(&mut self) -> PeekOption {
        match self.stream.peek() {
            Some(c) => Some((
                c,
                self.one_ahead_stream.peek(),
                self.two_ahead_stream.peek(),
            )),
            _ => None,
        }
    }

    /// Consume the next character from each stream and return the
    /// specified token.
    fn next_char_and_token(&mut self, token: Token) -> Token {
        self.next_char();
        token
    }

    /// Consume the next two characters from each stream and return the
    /// specified token.
    fn next_two_chars_and_token(&mut self, token: Token) -> Token {
        self.next_char();
        self.next_char();
        token
    }

    /// Consume and return the next character from each stream if the
    /// next character matches the specified condition.
    fn next_char_if(&mut self, func: impl FnOnce(&char) -> bool) -> NextOption {
        match self.stream.next_if(func) {
            Some(c) => {
                self.update_location(c);
                Some((
                    c,
                    self.one_ahead_stream.next(),
                    self.two_ahead_stream.next(),
                ))
            }
            _ => None,
        }
    }

    /// Consume the next two characters from each stream if the
    /// next two characters match their respective conditions. On match,
    /// the next two characters are returned.
    fn next_two_chars_if(
        &mut self,
        c_func: impl FnOnce(&char) -> bool,
        d_func: impl FnOnce(&char) -> bool,
    ) -> NextTwoOption {
        match (self.stream.peek(), self.one_ahead_stream.peek()) {
            (Some(c), Some(d)) => match c_func(c) && d_func(d) {
                true => {
                    let c = self.stream.next().unwrap();
                    let d = self.one_ahead_stream.next().unwrap();
                    let e = self.two_ahead_stream.next();
                    self.update_location(c);
                    self.next_char();
                    Some((c, d, e))
                }
                false => None,
            },
            _ => None,
        }
    }

    /// Consume the next three characters from each stream if the
    /// next three characters match their respective conditions. On
    /// match, the next three characters are returned.
    fn next_three_chars_if(
        &mut self,
        c_func: impl FnOnce(&char) -> bool,
        d_func: impl FnOnce(&char) -> bool,
        e_func: impl FnOnce(&char) -> bool,
    ) -> NextThreeOption {
        match (
            self.stream.peek(),
            self.one_ahead_stream.peek(),
            self.two_ahead_stream.peek(),
        ) {
            (Some(c), Some(d), Some(e)) => match c_func(c) && d_func(d) && e_func(e) {
                true => {
                    let c = self.stream.next().unwrap();
                    let d = self.one_ahead_stream.next().unwrap();
                    let e = self.two_ahead_stream.next().unwrap();
                    self.update_location(c);
                    self.next_char();
                    self.next_char();
                    Some((c, d, e))
                }
                false => None,
            },
            _ => None,
        }
    }

    /// Update line and column numbers *every* time a character is
    /// consumed from the stream.
    fn update_location(&mut self, c: char) {
        let (current_line, current_col) = (self.location.line, self.location.col);
        let (line, col) = match c {
            '\n' => (current_line + 1, 1),
            _ => (current_line, current_col + 1),
        };
        self.location = Location::new(line, col);
    }

    /// Consume contiguous whitespace up to the end of the line. Return
    /// the number of whitespace characters consumed.
    fn consume_whitespace(&mut self) -> usize {
        let mut count = 0;
        loop {
            match self.next_char_if(|&c| c.is_whitespace() && c != '\n') {
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
            match self.next_char_if(|&c| c == ' ') {
                Some(_) => count += 1,
                None => break count,
            }
        }
    }

    /// Read contiguous digits and an optional decimal point into a new
    /// string. If a dot is encountered, it will be included only if the
    /// char following the dot is another digit.
    fn read_number(&mut self, first_digit: char) -> (String, u32) {
        let mut string = String::new();

        let radix: u32 = match (first_digit, self.peek_char()) {
            ('0', Some(('b', _, _))) => 2,
            ('0', Some(('o', _, _))) => 8,
            ('0', Some(('x', _, _))) => 16,
            _ => 10,
        };

        if radix == 10 {
            string.push(first_digit);
        } else {
            self.next_char();
        }

        string.push_str(self.collect_digits(radix).as_str());

        if radix == 10 {
            match self.next_two_chars_if(|&c| c == '.', |&d| d.is_digit(radix)) {
                // If the number is followed by a dot and at least one
                // digit consume the dot, the digit, and any following
                // digits.
                Some((dot, digit, _)) => {
                    string.push(dot);
                    string.push(digit);
                    string.push_str(self.collect_digits(radix).as_str());
                }
                _ => (),
            }
            // Handle E notation *without* sign.
            match self.next_two_chars_if(|&c| c == 'e' || c == 'E', |&e| e.is_digit(radix)) {
                Some((_, digit, _)) => {
                    string.push('E');
                    string.push('+');
                    string.push(digit);
                    string.push_str(self.collect_digits(radix).as_str());
                }
                _ => (),
            }
            // Handle E notation *with* sign.
            match self.next_three_chars_if(
                |&c| c == 'e' || c == 'E',
                |&d| d == '+' || d == '-',
                |&e| e.is_digit(10),
            ) {
                Some((_, sign, digit)) => {
                    string.push('E');
                    string.push(sign);
                    string.push(digit);
                    string.push_str(self.collect_digits(radix).as_str());
                }
                _ => (),
            }
        }

        (string, radix)
    }

    fn collect_digits(&mut self, radix: u32) -> String {
        let mut digits = String::new();
        loop {
            match self.next_char_if(|&c| c.is_digit(radix)) {
                Some((digit, _, _)) => digits.push(digit),
                None => break digits,
            }
            match self.peek_char() {
                Some(('_', Some(c), _)) if c.is_digit(radix) => {
                    self.next_char();
                }
                _ => (),
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
            match self.next_char() {
                // Skip newlines preceded by backslash
                Some(('\\', Some('\n'), _)) => {
                    self.next_char();
                }
                // Handle embedded/escaped quotes
                Some(('\\', Some(d), _)) if d == quote => {
                    self.next_char();
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
            match self.next_char_if(|&c| c != '\n') {
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
            match self.next_char_if(|&c| c.is_ascii_lowercase() || c.is_digit(10) || c == '_') {
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
            match self.next_char_if(|&c| c.is_ascii_alphabetic() || c.is_digit(10)) {
                Some((c, _, _)) => string.push(c),
                None => break string,
            }
        }
    }
}

impl Iterator for Scanner<'_> {
    type Item = ScanResult;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.next_token();
        match result {
            Ok(TokenWithLocation {
                token: Token::EndOfInput,
                start: _,
                end: _,
            }) => None,
            Ok(t) => Some(Ok(t)),
            Err(t) => Some(Err(t)),
        }
    }
}
