use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};
use std::iter::Peekable;
use std::str::Chars;

use crate::scanner::location::Location;
use crate::util::{Source, Stack};

use super::result::{ScanError, ScanErrorType, ScanResult};
use super::token::{Token, TokenWithLocation};

type NextOption = Option<(char, Option<char>, Option<char>)>;
type NextTwoOption = Option<(char, char, Option<char>)>;
type NextThreeOption = Option<(char, char, char)>;
type PeekOption<'a> = Option<(&'a char, Option<&'a char>, Option<&'a char>)>;

/// Create a scanner with the specified text source, scan the text, and
/// return the resulting tokens or error.
pub fn scan(text: &str) -> Result<Vec<TokenWithLocation>, ScanError> {
    let cursor = Cursor::new(text);
    let mut scanner = Scanner::new(cursor, Location::new(1, 1));
    scanner.collect()
}

pub fn scan_file(file_name: &str) -> Result<Vec<TokenWithLocation>, ScanError> {
    let file = File::open(file_name).unwrap();
    let mut reader = BufReader::new(file);
    let mut scanner = Scanner::new(reader, Location::new(1, 1));
    scanner.collect()
}

/// Scan and assume success.
fn scan_optimistic(text: &str) -> Vec<TokenWithLocation> {
    match scan(text) {
        Ok(tokens) => tokens,
        Err(err) => panic!("Scan failed unexpectedly: {:?}", err),
    }
}

struct Scanner<T>
where
    T: BufRead + Clone,
{
    /// Stream of input characters from input string
    stream: Source<T>,
    /// The same stream but one character ahead for easier lookaheads
    one_ahead_stream: Source<T>,
    /// The same stream but two characters ahead for easier lookaheads
    two_ahead_stream: Source<T>,
    location: Location,
    /// Temporary storage for tokens.
    queue: VecDeque<TokenWithLocation>,
    /// Keep track of whether we're at the start of a line so indents
    /// can be handled specially.
    at_line_start: bool,
    indent_level: i32,
    bracket_stack: Stack<(char, Location)>,
}

impl<T> Scanner<T>
where
    T: BufRead + Clone,
{
    /// Create a new scanner from source text.
    /// XXX: Not sure if it's useful to be able to pass line & col no.
    fn new(reader: T, location: Location) -> Self {
        let one_ahead_reader = reader.clone();
        let two_ahead_reader = reader.clone();
        let stream = Source::new(reader);
        let mut one_ahead_stream = Source::new(one_ahead_reader);
        let mut two_ahead_stream = Source::new(two_ahead_reader);
        one_ahead_stream.next();
        two_ahead_stream.next();
        two_ahead_stream.next();
        Scanner {
            stream,
            one_ahead_stream,
            two_ahead_stream,
            location,
            queue: VecDeque::new(),
            at_line_start: true,
            indent_level: -1,
            bracket_stack: Stack::new(),
        }
    }

    fn next_from_queue(&mut self) -> ScanResult {
        while self.queue.is_empty() {
            self.add_tokens_to_queue()?;
        }
        Ok(self.queue.pop_front().unwrap())
    }

    fn add_token_to_queue(
        &mut self,
        token: Token,
        start: Location,
        end_option: Option<Location>,
    ) {
        let end = match end_option {
            Some(end) => end,
            None => Location::new(self.location.line, self.location.col - 1),
        };
        let token_with_location = TokenWithLocation::new(token, start, end);
        self.queue.push_back(token_with_location);
    }

    fn handle_indents(&mut self) -> Result<(), ScanError> {
        let start = self.location;
        let mut num_spaces = self.read_indent();
        let mut whitespace_count = self.consume_whitespace();

        match self.stream.peek() {
            None | Some('\n') => {
                // Blank or whitespace-only line; skip it.
                return Ok(());
            }
            Some('#') => {
                // Line containing just a comment; skip it.
                self.consume_comment();
                return Ok(());
            }
            _ => (),
        }

        // Now we have 0 or more spaces followed by some other char.
        // First, make sure it's a valid indent.
        if num_spaces % 4 != 0 {
            return Err(ScanError::new(
                ScanErrorType::InvalidIndent(num_spaces),
                start,
            ));
        }

        // Next, make sure the indent isn't followed by additional non-
        // space whitespace, because that would be confusing.
        if whitespace_count > 0 {
            return Err(ScanError::new(ScanErrorType::WhitespaceAfterIndent, start));
        }

        // Now we have something that could be a valid indent. If the
        // indent level has increased, that signals the start of a
        // block. If it has decreased, that signals the end of a block,
        // and we may have to dedent multiple levels. If it stayed the
        // same, do nothing.
        let indent_level = num_spaces / 4;
        if indent_level == self.indent_level {
            return Ok(());
        } else if self.indent_level == -1 {
            if indent_level == 0 {
                self.indent_level = 0;
                return Ok(());
            }
            return Err(ScanError::new(
                ScanErrorType::UnexpectedIndent(indent_level),
                start,
            ));
        } else if indent_level == self.indent_level + 1 {
            let location = Location::new(start.line, 0);
            self.indent_level = indent_level;
            self.add_token_to_queue(Token::BlockStart, location, Some(location));
        } else if indent_level < self.indent_level {
            let location = Location::new(start.line, 0);
            while self.indent_level > indent_level {
                self.indent_level -= 1;
                self.add_token_to_queue(Token::BlockEnd, location, Some(location));
            }
        } else {
            return Err(ScanError::new(
                ScanErrorType::UnexpectedIndent(indent_level),
                start,
            ));
        };

        Ok(())
    }

    fn add_tokens_to_queue(&mut self) -> Result<(), ScanError> {
        if self.at_line_start {
            self.handle_indents()?;
        }

        let start = self.location;

        // NOTE: handle_indents() may consume up to the end of a line,
        // but it will always leave the newline in place. We assume
        // that we're not at the start of a newline, but if the next
        // char is a newline, this will be updated below.
        self.at_line_start = false;

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
            Some(('#', _, _)) => {
                self.consume_comment();
                return Ok(());
            }
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
            Some(('<', Some('='), _)) => {
                self.next_char_and_token(Token::LessThanOrEqual)
            }
            Some(('<', Some('-'), _)) => self.next_char_and_token(Token::LoopFeed),
            Some((c @ '<', _, _)) => {
                self.bracket_stack.push((c, start));
                Token::LeftAngleBracket
            }
            Some(('>', Some('='), _)) => {
                self.next_char_and_token(Token::GreaterThanOrEqual)
            }
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
            Some(('!', _, _)) => Token::Bang,
            Some(('.', Some('.'), Some('.'))) => {
                self.next_two_chars_and_token(Token::RangeInclusive)
            }
            Some(('.', Some('.'), _)) => self.next_char_and_token(Token::Range),
            Some(('.', _, _)) => Token::Dot,
            Some(('%', _, _)) => Token::Percent,
            Some(('^', _, _)) => Token::Caret,
            Some((c @ '0'..='9', _, _)) => match self.read_number(c) {
                (string, _) if string.contains(".") || string.contains("E") => {
                    Token::Float(string)
                }
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
            Some((c @ 'A'..='Z', _, _)) => {
                Token::TypeIdentifier(self.read_type_identifier(c))
            }
            Some((c @ '@', Some('a'..='z'), _)) => {
                Token::TypeMethodIdentifier(self.read_identifier(c))
            }
            Some((c @ '$', Some('a'..='z'), _)) => {
                Token::SpecialMethodIdentifier(self.read_identifier(c))
            }
            // Newlines
            Some(('\n', _, _)) => {
                if self.bracket_stack.size() > 0 {
                    self.consume_whitespace();
                } else {
                    self.at_line_start = true;
                }
                return Ok(());
            }
            Some(('\r', Some('\n'), _)) => {
                // On the next iteration, the post-newline logic above
                // will be invoked.
                return Ok(());
            }
            Some((c, _, _)) if c.is_whitespace() => {
                return Err(ScanError::new(ScanErrorType::UnexpectedWhitespace, start));
            }
            // Unknown
            Some((c, _, _)) => {
                return Err(ScanError::new(
                    ScanErrorType::UnexpectedCharacter(c),
                    start,
                ));
            }
            // End of input
            None => {
                if self.indent_level > 0 {
                    // This will happen if the source doesn't end with
                    // a newline.
                    let location = Location::new(start.line + 1, 0);
                    while self.indent_level > 0 {
                        self.indent_level -= 1;
                        self.add_token_to_queue(
                            Token::BlockEnd,
                            location,
                            Some(location),
                        );
                    }
                }

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

        self.add_token_to_queue(token, start, None);
        self.consume_whitespace();
        Ok(())
    }

    /// Consume and return the next character from each stream.
    fn next_char(&mut self) -> NextOption {
        match self.stream.next() {
            Some(c) => {
                self.update_location(c);
                Some((c, self.one_ahead_stream.next(), self.two_ahead_stream.next()))
            }
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
                Some((c, self.one_ahead_stream.next(), self.two_ahead_stream.next()))
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

    /// Consume comment characters up to newline.
    fn consume_comment(&mut self) {
        while self.next_char_if(|&c| c != '\n').is_some() {}
    }

    /// Returns the number of contiguous space characters at the start
    /// of a line. An indent is defined as 4*N space characters followed
    /// by a non-whitespace character.
    fn read_indent(&mut self) -> i32 {
        let mut count = 0;
        loop {
            match self.next_char_if(|&c| c == ' ') {
                Some(_) => count += 1,
                None => break count,
            }
        }
    }

    /// Read a number. Base 2, 8, 10, and 16 ints are supported as well
    /// as base 10 floats.
    fn read_number(&mut self, first_digit: char) -> (String, u32) {
        let mut string = String::new();

        let radix: u32 = if first_digit == '0' {
            match self.stream.peek() {
                Some('b') | Some('B') => 2,
                Some('o') | Some('O') => 8,
                Some('x') | Some('X') => 16,
                Some(t) if t.is_ascii_alphabetic() => {
                    panic!("Unsupported numeric type: {}", t);
                }
                _ => 10,
            }
        } else {
            10
        };

        if radix == 10 {
            string.push(first_digit);
        } else {
            // Skip leading zero *and* type char.
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
            match self
                .next_two_chars_if(|&c| c == 'e' || c == 'E', |&e| e.is_digit(radix))
            {
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
                |&e| e.is_digit(radix),
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
                None => {
                    match self.next_two_chars_if(|&c| c == '_', |&d| d.is_digit(radix))
                    {
                        Some((_, digit, _)) => digits.push(digit),
                        None => break digits,
                    }
                }
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
                // Handle chars escaped by a preceding \.
                Some(('\\', Some(c), _)) => {
                    // TODO: Handle \o, \u, \x, etc
                    self.next_char(); // Skip over the \.
                    match c {
                        // Skip newline when preceded by \ at end of
                        // line. Note that this is the case where an
                        // actual newline is embedded in a multiline
                        // string and not the case where the string
                        // "\n" was typed out. The "\n" case is handled
                        // below.
                        '\n' => (),

                        'a' => string.push('\x07'),
                        'b' => string.push('\x08'),
                        'f' => string.push('\x0c'),

                        // These next few lines might seem pointless,
                        // but they're replacing the escape sequence in
                        // the source text with the *actual* char in the
                        // Rust string.
                        '0' => string.push('\0'), // null
                        'n' => string.push('\n'), // line feed
                        'r' => string.push('\r'), // carriage return
                        't' => string.push('\t'), // horizontal tab

                        'v' => string.push('\x0b'), // vertical tab

                        '\\' => string.push('\\'),

                        // Unescape single quote. I'm not entirely sure
                        // what the point of this is, since only double
                        // quotes are used to quote strings, but it
                        // seems to be a standard (Python and Rust both
                        // do it).
                        '\'' => string.push('\''),

                        // This also seems to be a standard.
                        '\"' => string.push('\"'),

                        // Any other escaped char resolves to the
                        // original *escaped* version of itself.
                        c => {
                            string.push('\\');
                            string.push(c);
                        }
                    }
                }
                // Found closing quote; return string.
                Some((c, _, _)) if c == quote => break (string, true),
                // Append current char and continue.
                Some((c, _, _)) => string.push(c),
                // End of input reached without finding closing quote :(
                None => break (string, false),
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
            match self
                .next_char_if(|&c| c.is_ascii_lowercase() || c.is_digit(10) || c == '_')
            {
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

impl<T> Iterator for Scanner<T>
where
    T: BufRead + Clone,
{
    type Item = ScanResult;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_from_queue() {
            Ok(TokenWithLocation { token: Token::EndOfInput, .. }) => None,
            Ok(t) => Some(Ok(t)),
            Err(t) => Some(Err(t)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_empty() {
        let tokens = scan_optimistic("");
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn scan_int() {
        let tokens = scan_optimistic("123");
        assert_eq!(tokens.len(), 1);
        check_token(tokens.get(0), Token::Int("123".to_string(), 10), 1, 1, 1, 3);
    }

    #[test]
    fn scan_binary_number() {
        let tokens = scan_optimistic("0b11");
        assert_eq!(tokens.len(), 1);
        check_token(tokens.get(0), Token::Int("11".to_string(), 2), 1, 1, 1, 4);
    }

    #[test]
    fn scan_float() {
        let tokens = scan_optimistic("123.1");
        assert_eq!(tokens.len(), 1);
        check_token(tokens.get(0), Token::Float("123.1".to_string()), 1, 1, 1, 5);
    }

    #[test]
    fn scan_float_with_e_and_no_sign() {
        let tokens = scan_optimistic("123.1e1");
        assert_eq!(tokens.len(), 1);
        let expected = Token::Float("123.1E+1".to_string());
        check_token(tokens.get(0), expected, 1, 1, 1, 7);
    }

    #[test]
    fn scan_float_with_e_and_sign() {
        let tokens = scan_optimistic("123.1e+1");
        assert_eq!(tokens.len(), 1);
        let expected = Token::Float("123.1E+1".to_string());
        check_token(tokens.get(0), expected, 1, 1, 1, 8);
    }

    #[test]
    fn scan_string_with_embedded_quote() {
        // "\"abc"
        let source = "\"\\\"abc\"";
        let tokens = scan_optimistic(source);
        assert_eq!(tokens.len(), 1);
        check_string_token(tokens.get(0), "\"abc", 1, 1, 1, 7);
    }

    #[test]
    fn scan_string_with_newline() {
        // "abc
        // "
        let source = "\"abc\n\"";
        let tokens = scan_optimistic(source);
        assert_eq!(tokens.len(), 1);
        check_string_token(tokens.get(0), "abc\n", 1, 1, 2, 1);
    }

    #[test]
    fn scan_string_with_many_newlines() {
        // " a
        // b
        //
        // c
        //
        //
        //   "
        let source = "\" a\nb\n\nc\n\n\n  \"";
        let tokens = scan_optimistic(source);
        assert_eq!(tokens.len(), 1);
        check_string_token(tokens.get(0), " a\nb\n\nc\n\n\n  ", 1, 1, 7, 3);
    }

    #[test]
    fn scan_string_with_escaped_chars() {
        let tokens = scan_optimistic("\"\\0\\a\\b\\n\\'\\\"\"");
        assert_eq!(tokens.len(), 1);
        // NOTE: We could put a backslash before the single quote in
        //       the expected string, but Rust seems to treat \' and '
        //       as the same.
        check_string_token(tokens.get(0), "\0\x07\x08\n'\"", 1, 1, 1, 14);
    }

    #[test]
    fn scan_string_with_escaped_regular_char() {
        let tokens = scan_optimistic("\"ab\\c\"");
        assert_eq!(tokens.len(), 1);
        check_string_token(tokens.get(0), "ab\\c", 1, 1, 1, 6);
    }

    #[test]
    fn scan_string_unclosed() {
        let source = "\"abc";
        match scan(source) {
            Err(err) => match err {
                ScanError {
                    error: ScanErrorType::UnterminatedString(string),
                    location,
                } => {
                    assert_eq!(string, source.to_string());
                    assert_eq!(location, Location::new(1, 1));
                    let new_source = source.to_string() + "\"";
                    match scan(new_source.as_str()) {
                        Ok(tokens) => {
                            assert_eq!(tokens.len(), 1);
                            check_string_token(tokens.get(0), "abc", 1, 1, 1, 5);
                        }
                        _ => assert!(false),
                    }
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        };
    }

    #[test]
    fn scan_indents() {
        let source = "\
f (x) ->  # 1
    x     # 2
    1     # 3
          # 4
          # 5
g (y) ->  # 6
    y     # 7\
";
        let tokens = scan_optimistic(source);

        // Used to keep rustfmt from wrapping
        let mut token;

        // f
        token = Token::Identifier("f".to_string());
        check_token(tokens.get(0), token, 1, 1, 1, 1);
        check_token(tokens.get(1), Token::LeftParen, 1, 3, 1, 3);
        token = Token::Identifier("x".to_string());
        check_token(tokens.get(2), token, 1, 4, 1, 4);
        check_token(tokens.get(3), Token::RightParen, 1, 5, 1, 5);
        check_token(tokens.get(4), Token::FuncStart, 1, 7, 1, 8);
        check_token(tokens.get(5), Token::BlockStart, 2, 0, 2, 0);
        token = Token::Identifier("x".to_string());
        check_token(tokens.get(6), token, 2, 5, 2, 5);
        check_token(tokens.get(7), Token::Int("1".to_string(), 10), 3, 5, 3, 5);
        check_token(tokens.get(8), Token::BlockEnd, 6, 0, 6, 0);

        // g
        token = Token::Identifier("g".to_string());
        check_token(tokens.get(9), token, 6, 1, 6, 1);
        check_token(tokens.get(10), Token::LeftParen, 6, 3, 6, 3);
        token = Token::Identifier("y".to_string());
        check_token(tokens.get(11), token, 6, 4, 6, 4);
        check_token(tokens.get(12), Token::RightParen, 6, 5, 6, 5);
        check_token(tokens.get(13), Token::FuncStart, 6, 7, 6, 8);
        check_token(tokens.get(14), Token::BlockStart, 7, 0, 7, 0);
        token = Token::Identifier("y".to_string());
        check_token(tokens.get(15), token, 7, 5, 7, 5);
        check_token(tokens.get(16), Token::BlockEnd, 8, 0, 8, 0);
        assert!(tokens.get(17).is_none());
    }

    #[test]
    fn scan_unexpected_indent_on_first_line() {
        let source = "    abc = 1";
        match scan(source) {
            Ok(_) => assert!(false),
            Err(err) => match err {
                ScanError { error: ScanErrorType::UnexpectedIndent(1), location } => {
                    assert_eq!(location.line, 1);
                    assert_eq!(location.col, 1);
                }
                _ => assert!(false),
            },
        }
    }

    #[test]
    fn scan_brackets() {
        let source = "

a = [
   1,
# comment
  2,
]

# FIXME: This is an unexpected indent but the scanner doesn't detect that.
    b = 1
";
        let tokens = scan_optimistic(source);
        let mut token;
        for token in tokens {
            eprintln!("{}", token);
        }
        // assert_eq!(tokens.len(), 8);
        token = Token::Identifier("a".to_string());
        // check_token(tokens.get(0), token, 3, 1, 3, 1);
        // check_token(tokens.get(1), Token::Equal, 3, 3, 3, 3);
        // check_token(tokens.get(2), Token::LeftSquareBracket, 3, 5, 3, 5);
        // check_token(tokens.get(3), Token::Int("1".to_owned(), 10), 4, 4, 4, 4);
        // check_token(tokens.get(4), Token::Comma, 4, 5, 4, 5);
        // check_token(tokens.get(5), Token::Int("2".to_owned(), 10), 6, 3, 6, 3);
        // check_token(tokens.get(6), Token::Comma, 6, 4, 6, 4);
        // check_token(tokens.get(7), Token::RightSquareBracket, 7, 1, 7, 1);
        // assert!(tokens.get(8).is_none());
    }

    #[test]
    fn scan_unknown() {
        let source = "{";
        match scan(source) {
            Ok(tokens) => assert!(false),
            Err(err) => match err {
                ScanError {
                    error: ScanErrorType::UnexpectedCharacter(c),
                    location,
                } => {
                    assert_eq!(c, '{');
                    assert_eq!(location.line, 1);
                    assert_eq!(location.col, 1);
                }
                _ => assert!(false),
            },
        }
    }

    // Utilities -----------------------------------------------------------

    /// Check token returned by scanner against expected token.
    fn check_token(
        actual: Option<&TokenWithLocation>,
        expected: Token,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) {
        let start = Location::new(start_line, start_col);
        let end = Location::new(end_line, end_col);
        assert_eq!(actual, Some(&TokenWithLocation::new(expected, start, end)));
    }

    fn check_string_token(
        actual: Option<&TokenWithLocation>,
        expected_string: &str,
        expected_start_line: usize,
        expected_start_col: usize,
        expected_end_line: usize,
        expected_end_col: usize,
    ) {
        assert!(actual.is_some());
        match actual {
            Some(TokenWithLocation {
                token: Token::String(actual_string),
                start: Location { line: actual_start_line, col: actual_start_col },
                end: Location { line: actual_end_line, col: actual_end_col },
            }) => {
                assert_eq!(actual_string, expected_string);
                assert_eq!(actual_start_line, &expected_start_line);
                assert_eq!(actual_start_col, &expected_start_col);
                assert_eq!(actual_end_line, &expected_end_line);
                assert_eq!(actual_end_col, &expected_end_col);
            }
            _ => assert!(false),
        }
    }
}
