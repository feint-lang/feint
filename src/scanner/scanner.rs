use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Cursor};

use num_bigint::BigInt;
use num_traits::Num;

use crate::util::{Location, Source, Stack};

use super::keywords::KEYWORDS;
use super::result::{ScanErr, ScanErrKind, ScanResult, ScanTokensResult};
use super::token::{Token, TokenWithLocation};

type NextOption<'a> = Option<(char, Option<&'a char>, Option<&'a char>)>;
type NextTwoOption<'a> = Option<(char, char, Option<&'a char>)>;
type NextThreeOption = Option<(char, char, char)>;

/// Create a scanner from the specified text, scan the text, and return
/// the resulting tokens or error.
pub fn scan_text(text: &str, debug: bool) -> ScanTokensResult {
    let scanner = Scanner::<Cursor<&str>>::from_text(text);
    handle_result(scanner.collect(), debug)
}

/// Create a scanner from the specified file, scan its text, and return
/// the resulting tokens or error.
pub fn scan_file(file_path: &str, debug: bool) -> ScanTokensResult {
    let result = Scanner::<BufReader<File>>::from_file(file_path);
    let scanner = match result {
        Ok(scanner) => scanner,
        Err(err) => {
            return Err(ScanErr::new(
                ScanErrKind::CouldNotOpenSourceFile(
                    file_path.to_string(),
                    err.to_string(),
                ),
                Location::new(0, 0),
            ));
        }
    };
    handle_result(scanner.collect(), debug)
}

/// Create a scanner from stdin, scan the text into tokens, and return
/// the resulting tokens or error.
pub fn scan_stdin(debug: bool) -> ScanTokensResult {
    let scanner = Scanner::<BufReader<io::Stdin>>::from_stdin();
    handle_result(scanner.collect(), debug)
}

/// Scan text and assume success, returning tokens in unwrapped form.
/// Panic on error. Mainly useful for testing.
pub fn scan_optimistic(text: &str, debug: bool) -> Vec<TokenWithLocation> {
    match scan_text(text, debug) {
        Ok(tokens) => tokens,
        Err(err) => panic!("Scan failed unexpectedly: {:?}", err),
    }
}

fn handle_result(result: ScanTokensResult, debug: bool) -> ScanTokensResult {
    result.map(|tokens| {
        if debug {
            for token in tokens.iter() {
                eprintln!("{:?}", token);
            }
        }
        tokens
    })
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

    fn handle_indents(&mut self) -> Result<(), ScanErr> {
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
            return Err(ScanErr::new(ScanErrKind::InvalidIndent(num_spaces), start));
        }

        // Next, make sure the indent isn't followed by additional non-
        // space whitespace, because that would be confusing.
        if whitespace_count > 0 {
            return Err(ScanErr::new(ScanErrKind::WhitespaceAfterIndent, start));
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
    ) -> Result<(), ScanErr> {
        if self.indent_level.is_none() {
            // Special case for first line of code
            return if indent_level == 0 {
                self.indent_level = Some(0);
                Ok(())
            } else {
                Err(ScanErr::new(ScanErrKind::UnexpectedIndent(indent_level), start))
            };
        }

        let mut current_level = self.indent_level.unwrap();
        let location = Location::new(start.line, 0);

        if indent_level == current_level {
            // Stayed the same; nothing to do
        } else if indent_level == current_level + 1 {
            // Increased by one level
            self.indent_level = Some(indent_level);
            self.add_token_to_queue(Token::ScopeStart, location, Some(location));
        } else if indent_level < current_level {
            // Decreased by one or more levels
            while current_level > indent_level {
                self.add_token_to_queue(Token::ScopeEnd, location, Some(location));
                self.add_token_to_queue(
                    Token::EndOfStatement,
                    location,
                    Some(location),
                );
                current_level -= 1;
            }
            self.indent_level = Some(current_level);
        } else {
            // Increased by *more* than one level
            return Err(ScanErr::new(
                ScanErrKind::UnexpectedIndent(indent_level),
                start,
            ));
        }

        Ok(())
    }

    fn add_tokens_to_queue(&mut self) -> Result<(), ScanErr> {
        let start = self.source.location();

        let token = match self.next_char() {
            Some(('"', _, _)) => match self.read_string('"') {
                (string, true) => Token::String(string),
                (string, false) => {
                    return Err(ScanErr::new(
                        ScanErrKind::UnterminatedString(format!("\"{}", string)),
                        start,
                    ));
                }
            },
            Some(('$', Some('"'), _)) => {
                self.next_char();
                match self.read_string('"') {
                    (string, true) => Token::FormatString(string),
                    (string, false) => {
                        return Err(ScanErr::new(
                            ScanErrKind::UnterminatedString(format!("$\"{}", string)),
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
                Token::LParen
            }
            Some((c @ ')', _, _)) => {
                self.pop_bracket_and_return_token(c, start, Token::RParen)?
            }
            Some(('[', _, _)) => {
                self.bracket_stack.push(('[', start));
                Token::LBracket
            }
            Some((c @ ']', _, _)) => {
                self.pop_bracket_and_return_token(c, start, Token::RBracket)?
            }
            Some(('<', Some('='), _)) => {
                self.consume_char_and_return_token(Token::LessThanOrEqual)
            }
            Some(('<', Some('-'), _)) => {
                self.consume_char_and_return_token(Token::LoopFeed)
            }
            Some(('<', _, _)) => {
                self.bracket_stack.push(('<', start));
                Token::LessThan
            }
            Some(('>', Some('='), _)) => {
                self.consume_char_and_return_token(Token::GreaterThanOrEqual)
            }
            Some((c @ '>', _, _)) => {
                self.pop_bracket_and_return_token(c, start, Token::GreaterThan)?
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
                        ScanErr::new(ScanErrKind::ParseFloatError(err), start)
                    })?;
                    Token::Float(value)
                }
                (string, radix) => {
                    let value = BigInt::from_str_radix(string.as_str(), radix)
                        .map_err(|err| {
                            ScanErr::new(ScanErrKind::ParseIntError(err), start)
                        })?;
                    Token::Int(value)
                }
            },
            // Identifiers
            // Special case for single underscore placeholder var
            Some(('_', _, _)) => {
                if self.consume_contiguous('_') > 1 {
                    return Err(ScanErr::new(
                        ScanErrKind::UnexpectedCharacter('_'),
                        Location::new(start.line, start.col + 1),
                    ));
                }
                Token::Ident("_".to_owned())
            }
            Some((c @ 'a'..='z', _, _)) => {
                let ident = self.read_ident(c);
                let items = (&self.previous_token, self.source.peek());
                if let (Token::EndOfStatement, Some(&':')) = items {
                    self.next_char();
                    Token::Label(ident)
                } else if let (Token::ScopeStart, Some(&':')) = items {
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
                return Err(ScanErr::new(ScanErrKind::UnexpectedWhitespace, start));
            }
            // Unknown
            Some((c, _, _)) => {
                return Err(ScanErr::new(ScanErrKind::UnexpectedCharacter(c), start));
            }
            // End of input
            None => {
                if self.bracket_stack.size() == 0 {
                    self.maybe_add_end_of_statement_token(start);
                    self.set_indent_level(0, Location::new(start.line + 1, 1))?;
                } else if let Some((c, location)) = self.bracket_stack.pop() {
                    return Err(ScanErr::new(
                        ScanErrKind::UnmatchedOpeningBracket(c),
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
    ) -> Result<Token, ScanErr> {
        match (self.bracket_stack.pop(), closing_bracket) {
            | (Some(('(', _)), ')')
            | (Some(('[', _)), ']')
            | (Some(('<', _)), '>')
            => {
                Ok(token)
            }
            _ => Err(ScanErr::new(
                ScanErrKind::UnmatchedClosingBracket(closing_bracket),
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
    fn consume_whitespace(&mut self) -> Result<u8, ScanErr> {
        let mut count = 0;
        loop {
            match self.next_char_if(|&c| c.is_whitespace() && c != '\n') {
                Some(_) => {
                    if count == u8::MAX {
                        return Err(ScanErr::new(
                            ScanErrKind::TooMuchWhitespace,
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
    fn read_indent(&mut self) -> Result<u8, ScanErr> {
        let mut count = 0;
        loop {
            match self.next_char_if(|&c| c == ' ') {
                Some(_) => {
                    if count == u8::MAX {
                        return Err(ScanErr::new(
                            ScanErrKind::TooMuchWhitespace,
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
