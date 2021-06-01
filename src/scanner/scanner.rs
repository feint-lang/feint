use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Cursor};

use num_bigint::BigInt;
use num_traits::Num;

use crate::util::{Location, Source, Stack};

use super::KEYWORDS;
use super::{ScanError, ScanErrorKind, ScanResult};
use super::{Token, TokenWithLocation};

type NextOption<'a> = Option<(char, Option<&'a char>, Option<&'a char>)>;
type NextTwoOption<'a> = Option<(char, char, Option<&'a char>)>;
type NextThreeOption = Option<(char, char, char)>;

/// Create a scanner from the specified text, scan the text, and return
/// the resulting tokens or error.
pub fn scan_text(text: &str) -> Result<Vec<TokenWithLocation>, ScanError> {
    let scanner = Scanner::<Cursor<&str>>::from_text(text);
    let tokens = scanner.collect();
    tokens
}

/// Create a scanner from the specified file, scan its text, and return
/// the resulting tokens or error.
pub fn scan_file(file_path: &str) -> Result<Vec<TokenWithLocation>, ScanError> {
    let result = Scanner::<BufReader<File>>::from_file(file_path);
    let scanner = match result {
        Ok(scanner) => scanner,
        Err(err) => {
            return Err(ScanError::new(
                ScanErrorKind::CouldNotOpenSourceFile(
                    file_path.to_string(),
                    err.to_string(),
                ),
                Location::new(0, 0),
            ));
        }
    };
    scanner.collect()
}

/// Create a scanner from stdin, scan the text into tokens, and return
/// the resulting tokens or error.
pub fn scan_stdin() -> Result<Vec<TokenWithLocation>, ScanError> {
    let scanner = Scanner::<BufReader<io::Stdin>>::from_stdin();
    scanner.collect()
}

/// Scan text and assume success, returning tokens in unwrapped form.
/// Panic on error. Mainly useful for testing.
pub fn scan_optimistic(text: &str) -> Vec<TokenWithLocation> {
    match scan_text(text) {
        Ok(tokens) => tokens,
        Err(err) => panic!("Scan failed unexpectedly: {:?}", err),
    }
}

pub struct Scanner<T: BufRead> {
    /// This is the source code that's being scanned. T can be anything
    /// that implements the BufRead trait (e.g., a Cursor wrapping some
    /// text or a BufReader wrapping an open file).
    source: Source<T>,
    /// Temporary storage for tokens. This is mainly needed to handle
    /// the complexity of indents, because there are cases where
    /// multiple tokens will need to be emitted.
    queue: VecDeque<TokenWithLocation>,
    /// Keep track of whether we're at the start of a line so indents
    /// can be handled specially.
    indent_level: Option<u8>,
    /// Opening brackets are pushed and later popped when the closing
    /// bracket is encountered. This gives us a way to verify brackets
    /// are matched and also lets us know when we're inside a group
    /// where leading whitespace can be ignored.
    bracket_stack: Stack<(char, Location)>,
    /// The last token that was popped from the queue.
    previous_token: Token,
}

impl<T: BufRead> Scanner<T> {
    fn new(reader: T) -> Self {
        Scanner {
            source: Source::new(reader),
            queue: VecDeque::new(),
            indent_level: None,
            bracket_stack: Stack::new(),
            previous_token: Token::EndOfStatement,
        }
    }

    /// Create a scanner from the specified text.
    pub fn from_text(text: &str) -> Scanner<Cursor<&str>> {
        let cursor = Cursor::new(text);
        let scanner = Scanner::new(cursor);
        scanner
    }

    /// Create a scanner from the specified file.
    pub fn from_file(file_path: &str) -> Result<Scanner<BufReader<File>>, io::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let scanner = Scanner::new(reader);
        Ok(scanner)
    }

    /// Create a scanner from stdin
    pub fn from_stdin() -> Scanner<BufReader<io::Stdin>> {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        let scanner = Scanner::new(reader);
        scanner
    }

    fn next_token_from_queue(&mut self) -> ScanResult {
        while self.queue.is_empty() {
            self.add_tokens_to_queue()?;
        }
        let token = self.queue.pop_front().unwrap();
        self.previous_token = token.token.clone();
        Ok(token)
    }

    fn handle_indents(&mut self) -> Result<(), ScanError> {
        assert_eq!(
            self.source.current_char,
            Some('\n'),
            "This method should only be called when at the start of a line"
        );

        let start = self.source.location();
        let num_spaces = self.read_indent()?;
        let whitespace_count = self.consume_whitespace()?;

        match self.source.peek() {
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
            return Err(ScanError::new(ScanErrorKind::InvalidIndent(num_spaces), start));
        }

        // Next, make sure the indent isn't followed by additional non-
        // space whitespace, because that would be confusing.
        if whitespace_count > 0 {
            return Err(ScanError::new(ScanErrorKind::WhitespaceAfterIndent, start));
        }

        // Now we have something that *could* be a valid indent.
        self.set_indent_level(num_spaces / 4, start)
    }

    /// Maybe update the current indent level. If the new indent level
    /// is the same as the current indent level, do nothing. If it has
    /// increased, that signals the start of a block. If it has
    /// decreased, that signals the end of one or more blocks.
    fn set_indent_level(
        &mut self,
        indent_level: u8,
        start: Location,
    ) -> Result<(), ScanError> {
        if self.indent_level.is_none() {
            // Special case for first line of code
            return if indent_level == 0 {
                self.indent_level = Some(0);
                Ok(())
            } else {
                Err(ScanError::new(ScanErrorKind::UnexpectedIndent(indent_level), start))
            };
        }

        let mut current_level = self.indent_level.unwrap();
        let location = Location::new(start.line, 0);

        if indent_level == current_level {
            // Stayed the same; nothing to do
        } else if indent_level == current_level + 1 {
            // Increased by one level
            self.indent_level = Some(indent_level);
            self.add_token_to_queue(Token::BlockStart, location, Some(location));
        } else if indent_level < current_level {
            // Decreased by one or more levels
            while current_level > indent_level {
                self.add_token_to_queue(Token::BlockEnd, location, Some(location));
                self.add_token_to_queue(Token::EndOfStatement, location, Some(location));
                current_level -= 1;
            }
            self.indent_level = Some(current_level);
        } else {
            // Increased by *more* than one level
            return Err(ScanError::new(
                ScanErrorKind::UnexpectedIndent(indent_level),
                start,
            ));
        }

        Ok(())
    }

    fn add_tokens_to_queue(&mut self) -> Result<(), ScanError> {
        let start = self.source.location();

        let token = match self.next_char() {
            Some(('"', _, _)) => match self.read_string('"') {
                (string, true) => Token::String(string),
                (string, false) => {
                    return Err(ScanError::new(
                        ScanErrorKind::UnterminatedString(format!("\"{}", string)),
                        start,
                    ));
                }
            },
            Some(('$', Some('"'), _)) => {
                self.next_char();
                match self.read_string('"') {
                    (string, true) => Token::FormatString(string),
                    (string, false) => {
                        return Err(ScanError::new(
                            ScanErrorKind::UnterminatedString(format!("$\"{}", string)),
                            start,
                        ));
                    }
                }
            }
            Some(('#', _, _)) => {
                self.consume_comment();
                return Ok(());
            }
            Some((':', _, _)) => Token::Colon,
            Some((',', _, _)) => Token::Comma,
            Some(('(', _, _)) => {
                self.bracket_stack.push(('(', start));
                Token::LeftParen
            }
            Some((c @ ')', _, _)) => {
                self.pop_bracket_and_return_token(c, start, Token::RightParen)?
            }
            Some(('[', _, _)) => {
                self.bracket_stack.push(('[', start));
                Token::LeftSquareBracket
            }
            Some((c @ ']', _, _)) => {
                self.pop_bracket_and_return_token(c, start, Token::RightSquareBracket)?
            }
            Some(('<', Some('='), _)) => {
                self.consume_char_and_return_token(Token::LessThanOrEqual)
            }
            Some(('<', Some('-'), _)) => {
                self.consume_char_and_return_token(Token::LoopFeed)
            }
            Some(('<', _, _)) => {
                self.bracket_stack.push(('<', start));
                Token::LeftAngleBracket
            }
            Some(('>', Some('='), _)) => {
                self.consume_char_and_return_token(Token::GreaterThanOrEqual)
            }
            Some((c @ '>', _, _)) => {
                self.pop_bracket_and_return_token(c, start, Token::RightAngleBracket)?
            }
            Some(('=', Some('='), _)) => {
                self.consume_char_and_return_token(Token::EqualEqual)
            }
            Some(('=', _, _)) => Token::Equal,
            Some(('&', Some('&'), _)) => self.consume_char_and_return_token(Token::And),
            Some(('&', _, _)) => self.consume_char_and_return_token(Token::Ampersand),
            Some(('|', Some('|'), _)) => self.consume_char_and_return_token(Token::Or),
            Some(('|', _, _)) => self.consume_char_and_return_token(Token::Pipe),
            Some(('*', Some('*'), _)) => {
                self.consume_char_and_return_token(Token::DoubleStar)
            }
            Some(('*', Some('='), _)) => {
                self.consume_char_and_return_token(Token::MulEqual)
            }
            Some(('*', _, _)) => Token::Star,
            Some(('/', Some('='), _)) => {
                self.consume_char_and_return_token(Token::DivEqual)
            }
            Some(('/', Some('/'), _)) => {
                self.consume_char_and_return_token(Token::DoubleSlash)
            }
            Some(('/', _, _)) => Token::Slash,
            Some(('+', Some('='), _)) => {
                self.consume_char_and_return_token(Token::PlusEqual)
            }
            Some(('+', _, _)) => {
                // Collapse contiguous plus signs down to a single +.
                // This is safe because + is effectively a no-op.
                self.consume_contiguous('+');
                Token::Plus
            }
            Some(('-', Some('='), _)) => {
                self.consume_char_and_return_token(Token::MinusEqual)
            }
            Some(('-', Some('>'), _)) => {
                self.consume_char_and_return_token(Token::FuncStart)
            }
            Some(('-', _, _)) => Token::Minus,
            Some(('!', Some('='), _)) => {
                self.consume_char_and_return_token(Token::NotEqual)
            }
            Some(('!', _, _)) => {
                // Collapse contiguous bangs down to a single ! or !!.
                // This is mainly to ensure !!!x is interpreted as
                // !(!!(x)) instead of !!(!(x)).
                let count = self.consume_contiguous('!');
                match count % 2 {
                    0 => Token::BangBang,
                    1 => Token::Bang,
                    _ => unreachable!(),
                }
            }
            Some(('.', Some('.'), Some('.'))) => {
                self.consume_two_chars_and_return_token(Token::RangeInclusive)
            }
            Some(('.', Some('.'), _)) => {
                self.consume_char_and_return_token(Token::Range)
            }
            Some(('.', _, _)) => Token::Dot,
            Some(('%', _, _)) => Token::Percent,
            Some(('^', _, _)) => Token::Caret,
            Some((c @ '0'..='9', _, _)) => match self.read_number(c) {
                (string, _) if string.contains(".") || string.contains("E") => {
                    let value = string.parse::<f64>().map_err(|err| {
                        ScanError::new(ScanErrorKind::ParseFloatError(err), start)
                    })?;
                    Token::Float(value)
                }
                (string, radix) => {
                    let value = BigInt::from_str_radix(string.as_str(), radix).map_err(
                        |err| ScanError::new(ScanErrorKind::ParseIntError(err), start),
                    )?;
                    Token::Int(value)
                }
            },
            // Identifiers
            Some((c @ 'a'..='z', _, _)) => {
                let ident = self.read_ident(c);
                let items = (&self.previous_token, self.source.peek());
                if let (Token::EndOfStatement, Some(&':')) = items {
                    self.next_char();
                    Token::Label(ident)
                } else {
                    match KEYWORDS.get(ident.as_str()) {
                        Some(token) => token.clone(),
                        _ => Token::Ident(ident),
                    }
                }
            }
            Some((c @ 'A'..='Z', _, _)) => Token::TypeIdent(self.read_type_ident(c)),
            Some((c @ '@', Some('a'..='z'), _)) => {
                Token::TypeMethodIdent(self.read_ident(c))
            }
            Some((c @ '$', Some('a'..='z'), _)) => {
                Token::SpecialMethodIdent(self.read_ident(c))
            }
            // Newlines
            Some(('\n', _, _)) => {
                if self.bracket_stack.size() == 0 {
                    self.maybe_add_end_of_statement_token(start);
                    self.handle_indents()?;
                } else {
                    self.consume_whitespace()?;
                }
                return Ok(());
            }
            Some((c, _, _)) if c.is_whitespace() => {
                return Err(ScanError::new(ScanErrorKind::UnexpectedWhitespace, start));
            }
            // Unknown
            Some((c, _, _)) => {
                return Err(ScanError::new(
                    ScanErrorKind::UnexpectedCharacter(c),
                    start,
                ));
            }
            // End of input
            None => {
                if self.bracket_stack.size() == 0 {
                    self.maybe_add_end_of_statement_token(start);
                    self.set_indent_level(0, Location::new(start.line + 1, 1))?;
                } else if let Some((c, location)) = self.bracket_stack.pop() {
                    return Err(ScanError::new(
                        ScanErrorKind::UnmatchedOpeningBracket(c),
                        location,
                    ));
                }
                Token::EndOfInput
            }
        };

        self.add_token_to_queue(token, start, None);
        self.consume_whitespace()?;
        Ok(())
    }

    fn maybe_add_end_of_statement_token(&mut self, location: Location) -> bool {
        if self.previous_token == Token::EndOfStatement {
            return false;
        }
        if self.bracket_stack.size() > 0 {
            return false;
        }
        self.add_token_to_queue(Token::EndOfStatement, location, Some(location));
        true
    }

    fn add_token_to_queue(
        &mut self,
        token: Token,
        start: Location,
        end_option: Option<Location>,
    ) {
        let end = match end_option {
            Some(end) => end,
            None => Location::new(
                self.source.line,
                if self.source.col == 0 { 0 } else { self.source.col - 1 },
            ),
        };
        let token_with_location = TokenWithLocation::new(token, start, end);
        self.queue.push_back(token_with_location);
    }

    /// Consume the next character and return the specified token.
    fn consume_char_and_return_token(&mut self, token: Token) -> Token {
        self.next_char();
        token
    }

    /// Consume the next two characters and return the specified token.
    fn consume_two_chars_and_return_token(&mut self, token: Token) -> Token {
        self.next_char();
        self.next_char();
        token
    }

    /// Check the specified closing bracket to ensure the last opening
    /// bracket matches. If it does, the specified token is returned.
    #[rustfmt::skip]
    fn pop_bracket_and_return_token(
        &mut self,
        closing_bracket: char,
        location: Location,
        token: Token,
    ) -> Result<Token, ScanError> {
        match (self.bracket_stack.pop(), closing_bracket) {
            | (Some(('(', _)), ')')
            | (Some(('[', _)), ']')
            | (Some(('<', _)), '>')
            => {
                Ok(token)
            }
            _ => Err(ScanError::new(
                ScanErrorKind::UnmatchedClosingBracket(closing_bracket),
                location,
            ))
        }
    }

    /// Consume and return the next character. The following two
    /// characters are included as well for easy peeking.
    fn next_char(&mut self) -> NextOption {
        match self.source.next() {
            Some(c) => {
                let (d, e) = self.source.peek_2();
                Some((c, d, e))
            }
            _ => None,
        }
    }

    /// Consume and return the next character if it matches the
    /// specified condition.
    fn next_char_if(&mut self, func: impl FnOnce(&char) -> bool) -> NextOption {
        if let Some(c) = self.source.peek() {
            if func(c) {
                let c = self.source.next().unwrap();
                let (d, e) = self.source.peek_2();
                return Some((c, d, e));
            }
        }
        None
    }

    /// Consume and return the next two characters if the next two
    /// characters match their respective conditions.
    fn next_two_chars_if(
        &mut self,
        c_func: impl FnOnce(&char) -> bool,
        d_func: impl FnOnce(&char) -> bool,
    ) -> NextTwoOption {
        match self.source.peek_2() {
            (Some(c), Some(d)) => match c_func(c) && d_func(d) {
                true => {
                    let c = self.source.next().unwrap();
                    let d = self.source.next().unwrap();
                    let e = self.source.peek();
                    Some((c, d, e))
                }
                false => None,
            },
            _ => None,
        }
    }

    /// Consume and return the next three characters if the next three
    /// characters match their respective conditions.
    fn next_three_chars_if(
        &mut self,
        c_func: impl FnOnce(&char) -> bool,
        d_func: impl FnOnce(&char) -> bool,
        e_func: impl FnOnce(&char) -> bool,
    ) -> NextThreeOption {
        let (c, d, e) = self.source.peek_3();
        match (c, d, e) {
            (Some(c), Some(d), Some(e)) => match c_func(c) && d_func(d) && e_func(e) {
                true => {
                    let c = self.source.next().unwrap();
                    let d = self.source.next().unwrap();
                    let e = self.source.next().unwrap();
                    Some((c, d, e))
                }
                false => None,
            },
            _ => None,
        }
    }

    /// Consume contiguous whitespace up to the end of the line. Return
    /// the number of whitespace characters consumed.
    fn consume_whitespace(&mut self) -> Result<u8, ScanError> {
        let mut count = 0;
        loop {
            match self.next_char_if(|&c| c.is_whitespace() && c != '\n') {
                Some(_) => {
                    if count == u8::MAX {
                        return Err(ScanError::new(
                            ScanErrorKind::TooMuchWhitespace,
                            self.source.location(),
                        ));
                    }
                    count += 1
                }
                None => break Ok(count),
            }
        }
    }

    /// Consume comment characters up to newline.
    fn consume_comment(&mut self) {
        while self.next_char_if(|&c| c != '\n').is_some() {}
    }

    /// Consume contiguous chars and return count.
    fn consume_contiguous(&mut self, c: char) -> u8 {
        let mut count = 1;
        while self.next_char_if(|&next_char| next_char == c).is_some() {
            count += 1;
        }
        count
    }

    /// Returns the number of contiguous space characters at the start
    /// of a line. An indent is defined as 4*N space characters followed
    /// by a non-whitespace character.
    fn read_indent(&mut self) -> Result<u8, ScanError> {
        let mut count = 0;
        loop {
            match self.next_char_if(|&c| c == ' ') {
                Some(_) => {
                    if count == u8::MAX {
                        return Err(ScanError::new(
                            ScanErrorKind::TooMuchWhitespace,
                            self.source.location(),
                        ));
                    }
                    count += 1;
                }
                None => break Ok(count),
            }
        }
    }

    /// Read a number. Base 2, 8, 10, and 16 ints are supported as well
    /// as base 10 floats.
    fn read_number(&mut self, first_digit: char) -> (String, u32) {
        let mut string = String::new();

        let radix: u32 = if first_digit == '0' {
            match self.source.peek() {
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
                    match self.next_two_chars_if(|&c| c == '_', |&d| d.is_digit(radix)) {
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
            if let Some((_, d, _)) = self.next_two_chars_if(|c| c == &'\\', |_d| true) {
                // Handle chars escaped by a preceding \.
                // TODO: Handle \o, \u, \x, etc
                match d {
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
                    other => {
                        string.push('\\');
                        string.push(other);
                    }
                }
            } else {
                match self.next_char() {
                    // Found closing quote; return string.
                    Some((c, _, _)) if c == quote => break (string, true),
                    // Append current char and continue.
                    Some((c, _, _)) => string.push(c),
                    // End of input reached without finding closing quote :(
                    None => break (string, false),
                }
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
    fn read_ident(&mut self, first_char: char) -> String {
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
    fn read_type_ident(&mut self, first_char: char) -> String {
        let mut string = first_char.to_string();
        loop {
            match self.next_char_if(|&c| c.is_ascii_alphabetic() || c.is_digit(10)) {
                Some((c, _, _)) => string.push(c),
                None => break string,
            }
        }
    }
}

impl<T: BufRead> Iterator for Scanner<T> {
    type Item = ScanResult;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token_from_queue() {
            Ok(TokenWithLocation { token: Token::EndOfInput, .. }) => None,
            Ok(token) => Some(Ok(token)),
            err => Some(err),
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
        assert_eq!(tokens.len(), 2);
        check_token(tokens.get(0), Token::Int(BigInt::from(123)), 1, 1, 1, 3);
        check_token(tokens.get(1), Token::EndOfStatement, 1, 4, 1, 4);
    }

    #[test]
    fn scan_binary_number() {
        let tokens = scan_optimistic("0b11"); // = 3
        assert_eq!(tokens.len(), 2);
        check_token(tokens.get(0), Token::Int(BigInt::from(3)), 1, 1, 1, 4);
        check_token(tokens.get(1), Token::EndOfStatement, 1, 5, 1, 5);
    }

    #[test]
    fn scan_float() {
        let tokens = scan_optimistic("123.1");
        assert_eq!(tokens.len(), 2);
        check_token(tokens.get(0), Token::Float(123.1 as f64), 1, 1, 1, 5);
        check_token(tokens.get(1), Token::EndOfStatement, 1, 6, 1, 6);
    }

    #[test]
    fn scan_float_with_e_and_no_sign() {
        let tokens = scan_optimistic("123.1e1");
        assert_eq!(tokens.len(), 2);
        let expected = Token::Float(123.1E+1);
        check_token(tokens.get(0), expected, 1, 1, 1, 7);
        check_token(tokens.get(1), Token::EndOfStatement, 1, 8, 1, 8);
    }

    #[test]
    fn scan_float_with_e_and_sign() {
        let tokens = scan_optimistic("123.1e+1");
        assert_eq!(tokens.len(), 2);
        let expected = Token::Float(123.1E+1);
        check_token(tokens.get(0), expected, 1, 1, 1, 8);
        check_token(tokens.get(1), Token::EndOfStatement, 1, 9, 1, 9);
    }

    #[test]
    fn scan_string_with_embedded_quote() {
        // "\"abc"
        let source = "\"\\\"abc\"";
        let tokens = scan_optimistic(source);
        assert_eq!(tokens.len(), 2);
        check_string_token(tokens.get(0), "\"abc", 1, 1, 1, 7);
        check_token(tokens.get(1), Token::EndOfStatement, 1, 8, 1, 8);
    }

    #[test]
    fn scan_string_with_newline() {
        // "abc
        // "
        let source = "\"abc\n\"";
        let tokens = scan_optimistic(source);
        assert_eq!(tokens.len(), 2);
        check_string_token(tokens.get(0), "abc\n", 1, 1, 2, 1);
        check_token(tokens.get(1), Token::EndOfStatement, 2, 2, 2, 2);
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
        assert_eq!(tokens.len(), 2);
        check_string_token(tokens.get(0), " a\nb\n\nc\n\n\n  ", 1, 1, 7, 3);
        check_token(tokens.get(1), Token::EndOfStatement, 7, 4, 7, 4);
    }

    #[test]
    fn scan_string_with_escaped_chars() {
        let tokens = scan_optimistic("\"\\0\\a\\b\\n\\'\\\"\"");
        assert_eq!(tokens.len(), 2);
        // NOTE: We could put a backslash before the single quote in
        //       the expected string, but Rust seems to treat \' and '
        //       as the same.
        check_string_token(tokens.get(0), "\0\x07\x08\n'\"", 1, 1, 1, 14);
        check_token(tokens.get(1), Token::EndOfStatement, 1, 15, 1, 15);
    }

    #[test]
    fn scan_string_with_escaped_regular_char() {
        let tokens = scan_optimistic("\"ab\\c\"");
        assert_eq!(tokens.len(), 2);
        check_string_token(tokens.get(0), "ab\\c", 1, 1, 1, 6);
        check_token(tokens.get(1), Token::EndOfStatement, 1, 7, 1, 7);
    }

    #[test]
    fn scan_string_unclosed() {
        let source = "\"abc";
        match scan_text(source) {
            Err(err) => match err {
                ScanError {
                    kind: ScanErrorKind::UnterminatedString(string),
                    location,
                } => {
                    assert_eq!(string, source.to_string());
                    assert_eq!(location, Location::new(1, 1));
                    let new_source = source.to_string() + "\"";
                    match scan_text(new_source.as_str()) {
                        Ok(tokens) => {
                            assert_eq!(tokens.len(), 2);
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
        let mut tokens = tokens.iter();

        // f
        check_token(tokens.next(), Token::Ident("f".to_string()), 1, 1, 1, 1);
        check_token(tokens.next(), Token::LeftParen, 1, 3, 1, 3);
        check_token(tokens.next(), Token::Ident("x".to_string()), 1, 4, 1, 4);
        check_token(tokens.next(), Token::RightParen, 1, 5, 1, 5);
        check_token(tokens.next(), Token::FuncStart, 1, 7, 1, 8);
        check_token(tokens.next(), Token::EndOfStatement, 1, 14, 1, 14);
        check_token(tokens.next(), Token::BlockStart, 2, 0, 2, 0);
        check_token(tokens.next(), Token::Ident("x".to_string()), 2, 5, 2, 5);
        check_token(tokens.next(), Token::EndOfStatement, 2, 14, 2, 14);
        check_token(tokens.next(), Token::Int(BigInt::from(1)), 3, 5, 3, 5);
        check_token(tokens.next(), Token::EndOfStatement, 3, 14, 3, 14);
        check_token(tokens.next(), Token::BlockEnd, 6, 0, 6, 0);
        check_token(tokens.next(), Token::EndOfStatement, 6, 0, 6, 0);

        // g
        check_token(tokens.next(), Token::Ident("g".to_string()), 6, 1, 6, 1);
        check_token(tokens.next(), Token::LeftParen, 6, 3, 6, 3);
        check_token(tokens.next(), Token::Ident("y".to_string()), 6, 4, 6, 4);
        check_token(tokens.next(), Token::RightParen, 6, 5, 6, 5);
        check_token(tokens.next(), Token::FuncStart, 6, 7, 6, 8);
        check_token(tokens.next(), Token::EndOfStatement, 6, 14, 6, 14);
        check_token(tokens.next(), Token::BlockStart, 7, 0, 7, 0);
        check_token(tokens.next(), Token::Ident("y".to_string()), 7, 5, 7, 5);
        check_token(tokens.next(), Token::EndOfStatement, 7, 14, 7, 14);
        check_token(tokens.next(), Token::BlockEnd, 8, 0, 8, 0);
        check_token(tokens.next(), Token::EndOfStatement, 8, 0, 8, 0);

        assert!(tokens.next().is_none());
    }

    #[test]
    fn scan_unexpected_indent_on_first_line() {
        let source = "    abc = 1";
        let result = scan_text(source);
        assert!(result.is_err());
        match result.unwrap_err() {
            ScanError { kind: ScanErrorKind::UnexpectedIndent(1), location } => {
                assert_eq!(location.line, 1);
                assert_eq!(location.col, 1);
            }
            err => assert!(false, "Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn scan_brackets() {
        let source = "

a = [
   1,
# comment
  2,
]  # another comment

b = 3
";
        let tokens = scan_optimistic(source);
        let mut tokens = tokens.iter();

        check_token(tokens.next(), Token::Ident("a".to_string()), 3, 1, 3, 1);
        check_token(tokens.next(), Token::Equal, 3, 3, 3, 3);
        check_token(tokens.next(), Token::LeftSquareBracket, 3, 5, 3, 5);
        check_token(tokens.next(), Token::Int(BigInt::from(1)), 4, 4, 4, 4);
        check_token(tokens.next(), Token::Comma, 4, 5, 4, 5);
        check_token(tokens.next(), Token::Int(BigInt::from(2)), 6, 3, 6, 3);
        check_token(tokens.next(), Token::Comma, 6, 4, 6, 4);
        check_token(tokens.next(), Token::RightSquareBracket, 7, 1, 7, 1);
        check_token(tokens.next(), Token::EndOfStatement, 7, 21, 7, 21);
        check_token(tokens.next(), Token::Ident("b".to_string()), 9, 1, 9, 1);
        check_token(tokens.next(), Token::Equal, 9, 3, 9, 3);
        check_token(tokens.next(), Token::Int(BigInt::from(3)), 9, 5, 9, 5);
        check_token(tokens.next(), Token::EndOfStatement, 9, 6, 9, 6);
        assert!(tokens.next().is_none());
    }

    #[test]
    fn scan_unknown() {
        let source = "{";
        match scan_text(source) {
            Ok(tokens) => assert!(false),
            Err(err) => match err {
                ScanError { kind: ScanErrorKind::UnexpectedCharacter(c), location } => {
                    assert_eq!(c, '{');
                    assert_eq!(location.line, 1);
                    assert_eq!(location.col, 1);
                }
                _ => assert!(false),
            },
        }
    }

    // Utilities -------------------------------------------------------

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
