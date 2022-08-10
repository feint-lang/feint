use std::collections::VecDeque;
use std::io::BufRead;

use num_bigint::BigInt;
use num_traits::Num;

use crate::format::scan_format_string;
use crate::scanner::result::AddTokenResult;
use crate::util::{Location, Source, Stack};

use super::keywords::KEYWORDS;
use super::result::ScanErrKind as ErrKind;
use super::result::{AddTokensResult, ScanErr, ScanTokenResult};
use super::token::{Token, TokenWithLocation};

type NextOption<'a> = Option<(char, Option<&'a char>, Option<&'a char>)>;
type NextTwoOption<'a> = Option<(char, char, Option<&'a char>)>;
type NextThreeOption = Option<(char, char, char)>;

pub struct Scanner<'a, T: BufRead> {
    /// This is the source code that's being scanned. T can be anything
    /// that implements the BufRead trait (e.g., a Cursor wrapping some
    /// text or a BufReader wrapping an open file).
    source: &'a mut Source<T>,
    /// Temporary storage for tokens. This is mainly needed to handle
    /// the complexity of indents, because there are cases where
    /// multiple tokens will need to be emitted.
    queue: VecDeque<TokenWithLocation>,
    /// Keep track of whether we're at the start of a line so indents
    /// can be handled specially.
    indent_level: u8,
    /// Opening brackets are pushed and later popped when the closing
    /// bracket is encountered. This gives us a way to verify brackets
    /// are matched and also lets us know when we're inside a group
    /// where leading whitespace can be ignored.
    bracket_stack: Stack<(char, Location)>,
    /// Stack to keep track of inline blocks (e.g., `block -> true`
    /// where there's no newline after the `->`).
    inline_scope_stack: Stack<Location>,
    /// Keep track of where `if`s are encountered. This is used with
    /// exit from certain inline blocks.
    /// TODO: Not sure this is the best way to handle this. Currently,
    ///       any `if` without an `else` will never be removed from this
    ///       stack.
    if_stack: Stack<Location>,
    /// The last token that was popped from the queue.
    last_token_from_queue: Token,
}

impl<'a, T: BufRead> Scanner<'a, T> {
    pub fn new(source: &'a mut Source<T>) -> Self {
        Scanner {
            source,
            queue: VecDeque::with_capacity(1024),
            indent_level: 0,
            bracket_stack: Stack::new(),
            inline_scope_stack: Stack::new(),
            if_stack: Stack::new(),
            last_token_from_queue: Token::EndOfStatement,
        }
    }

    /// Get the next token. If the token queue is empty, scanning will
    /// proceed from the current source location to refill the queue.
    /// When the end of the input is reached, an EndOfInput token is
    /// returned.
    fn next_token_from_queue(&mut self) -> ScanTokenResult {
        while self.queue.is_empty() {
            self.add_tokens_to_queue()?;
        }
        let token = self.queue.pop_front().unwrap();
        self.last_token_from_queue = token.token.clone();
        Ok(token)
    }

    /// Get the last token. If there are pending tokens in the queue,
    /// the last pending token will be returned. Otherwise, the last
    /// processed token will be returned.
    fn last_token(&self) -> &Token {
        match self.queue.back() {
            Some(t) => &t.token,
            None => &self.last_token_from_queue,
        }
    }

    /// Scan input starting from current source location and add one or
    /// more tokens to the queue.
    fn add_tokens_to_queue(&mut self) -> AddTokensResult {
        use ErrKind::*;
        use Token::*;

        let loc = self.source.loc();
        let start = Location::new(loc.line, loc.col + 1);

        let token = match self.next_char() {
            Some((quote @ ('"' | '\''), _, _)) => self.handle_string(quote, start)?,
            Some(('$', Some('"' | '\''), _)) => self.handle_format_string(start)?,
            Some(('#', _, _)) => {
                self.consume_comment();
                return Ok(());
            }
            Some((':', _, _)) => Colon,
            Some((',', _, _)) => {
                self.maybe_exit_inline_scope(start, false);
                Comma
            }
            Some(('(', _, _)) => {
                self.bracket_stack.push(('(', start));
                LParen
            }
            Some((c @ ')', _, _)) => {
                self.maybe_exit_inline_scope(start, false);
                self.pop_bracket_and_return_token(c, start, RParen)?
            }
            Some(('[', _, _)) => {
                self.bracket_stack.push(('[', start));
                LBracket
            }
            Some((c @ ']', _, _)) => {
                self.maybe_exit_inline_scope(start, false);
                self.pop_bracket_and_return_token(c, start, RBracket)?
            }
            Some(('{', _, _)) => {
                self.bracket_stack.push(('{', start));
                LBrace
            }
            Some((c @ '}', _, _)) => {
                self.maybe_exit_inline_scope(start, false);
                self.pop_bracket_and_return_token(c, start, RBrace)?
            }
            Some(('<', Some('='), _)) => {
                self.consume_char_and_return_token(LessThanOrEqual)
            }
            Some(('<', Some('-'), _)) => self.consume_char_and_return_token(LoopFeed),
            Some(('<', _, _)) => LessThan,
            Some(('>', Some('='), _)) => {
                self.consume_char_and_return_token(GreaterThanOrEqual)
            }
            Some(('>', _, _)) => GreaterThan,
            Some(('=', Some('='), Some('='))) => {
                self.consume_two_chars_and_return_token(EqualEqualEqual)
            }
            Some(('=', Some('='), _)) => self.consume_char_and_return_token(EqualEqual),
            Some(('=', _, _)) => Equal,
            Some(('&', Some('&'), _)) => self.consume_char_and_return_token(And),
            Some(('&', _, _)) => self.consume_char_and_return_token(Ampersand),
            Some(('|', Some('|'), _)) => self.consume_char_and_return_token(Or),
            Some(('|', _, _)) => self.consume_char_and_return_token(Pipe),
            Some(('*', Some('='), _)) => self.consume_char_and_return_token(MulEqual),
            Some(('*', _, _)) => Star,
            Some(('/', Some('='), _)) => self.consume_char_and_return_token(DivEqual),
            Some(('/', Some('/'), _)) => {
                self.consume_char_and_return_token(DoubleSlash)
            }
            Some(('/', _, _)) => Slash,
            Some(('+', Some('='), _)) => self.consume_char_and_return_token(PlusEqual),
            Some(('+', _, _)) => {
                // Collapse contiguous plus signs down to a single +.
                // This is safe because + is effectively a no-op.
                self.consume_contiguous('+');
                Plus
            }
            Some(('-', Some('='), _)) => self.consume_char_and_return_token(MinusEqual),
            Some(('-', Some('>'), _)) => {
                return self.handle_scope_start(start);
            }
            Some(('-', _, _)) => Minus,
            Some(('!', Some('='), Some('='))) => {
                self.consume_two_chars_and_return_token(NotEqualEqual)
            }
            Some(('!', Some('='), _)) => self.consume_char_and_return_token(NotEqual),
            Some(('!', _, _)) => self.handle_bang()?,
            Some(('.', Some('.'), Some('.'))) => {
                self.consume_two_chars_and_return_token(Ellipsis)
            }
            Some(('.', Some('.'), _)) => self.consume_char_and_return_token(DotDot),
            Some(('.', _, _)) => Dot,
            Some(('%', _, _)) => Percent,
            Some(('^', _, _)) => Caret,
            Some((c @ '0'..='9', _, _)) => self.handle_number(c, start)?,
            Some(('_', _, _)) => self.handle_ident('_', start)?,
            Some((c @ 'a'..='z', _, _)) => self.handle_ident(c, start)?,
            Some((c @ 'A'..='Z', _, _)) => TypeIdent(self.read_type_ident(c)),
            Some((c @ '@', Some('a'..='z'), _)) => TypeFuncIdent(self.read_ident(c)),
            Some((c @ '$', Some('a'..='z'), _)) => SpecialIdent(self.read_ident(c)),
            Some(('\n', _, _)) => return self.handle_newline(start),
            Some((c, _, _)) if c.is_whitespace() => {
                return Err(ScanErr::new(UnexpectedWhitespace, start));
            }
            Some((c, _, _)) => return Err(ScanErr::new(UnexpectedChar(c), start)),
            None => return self.handle_end_of_input(start),
        };

        let end = self.source.loc();
        self.add_token_to_queue(token, start, end);
        self.consume_whitespace();

        // The following ensures that if a token is followed by only
        // trailing whitespace and/or a comment that the EndOfStatement
        // token, if one is added, will have the correct location.
        if self.next_char_is('#') {
            self.consume_comment();
        }
        if self.next_char_is('\n') {
            let loc = Location::new(end.line, end.col + 1);
            self.handle_newline(loc)?;
        } else if self.source.peek().is_none() {
            let loc = Location::new(end.line, end.col + 1);
            self.handle_end_of_input(loc)?;
        }

        Ok(())
    }

    fn add_token_to_queue(&mut self, token: Token, start: Location, end: Location) {
        let token_with_location = TokenWithLocation::new(token, start, end);
        self.queue.push_back(token_with_location);
    }

    // Token Handlers --------------------------------------------------

    fn handle_bang(&mut self) -> AddTokenResult {
        // Collapse contiguous bangs down to a single ! or !!. This is
        // mainly to ensure !!!x is interpreted as !(!!(x)) instead of
        // !!(!(x)).
        let count = self.consume_contiguous('!') + 1;
        if count % 2 == 0 {
            Ok(Token::BangBang)
        } else {
            Ok(Token::Bang)
        }
    }

    fn handle_string(&mut self, quote: char, start: Location) -> AddTokenResult {
        let (string, terminated) = self.read_string(quote);
        if terminated {
            Ok(Token::Str(string))
        } else {
            Err(ScanErr::new(
                ErrKind::UnterminatedStr(format!("{quote}{string}")),
                start,
            ))
        }
    }

    fn handle_format_string(&mut self, start: Location) -> AddTokenResult {
        let quote = self.source.next().unwrap();
        let (string, terminated) = self.read_string(quote);
        if terminated {
            let format_string_tokens = scan_format_string(string.as_str())
                .map_err(|err| ScanErr::new(ErrKind::FormatStrErr(err), start))?;
            Ok(Token::FormatStr(format_string_tokens))
        } else {
            Err(ScanErr::new(
                ErrKind::UnterminatedStr(format!("${quote}{string}")),
                start,
            ))
        }
    }

    fn handle_number(&mut self, first_digit: char, start: Location) -> AddTokenResult {
        let (string, radix) = self.read_number(first_digit);
        let is_float = string.contains('.') || string.contains('E');
        if is_float {
            let value = string
                .parse::<f64>()
                .map_err(|err| ScanErr::new(ErrKind::ParseFloatErr(err), start))?;
            Ok(Token::Float(value))
        } else {
            let value = BigInt::from_str_radix(string.as_str(), radix)
                .map_err(|err| ScanErr::new(ErrKind::ParseIntErr(err), start))?;
            Ok(Token::Int(value))
        }
    }

    fn handle_ident(&mut self, first_char: char, start: Location) -> AddTokenResult {
        use Token::{
            Else, EndOfStatement, Ident, If, InlineScopeStart, Label, ScopeStart,
        };
        // Special case for underscore placeholder vars.
        if first_char == '_' {
            let count = self.consume_contiguous('_') + 1;
            let mut ident = String::with_capacity(count as usize);
            for _ in 0..count {
                ident.push('_');
            }
            return Ok(Ident(ident));
        }
        let ident = self.read_ident(first_char);
        // Label
        if let EndOfStatement | ScopeStart | InlineScopeStart = self.last_token() {
            if let Some(':') = self.source.peek() {
                self.source.next();
                return Ok(Label(ident));
            }
        }
        // Keyword
        if let Some(token) = KEYWORDS.get(ident.as_str()) {
            if token == &If {
                self.if_stack.push(start);
            } else if token == &Else {
                if self.maybe_exit_inline_scope(start, true) {
                    self.add_token_to_queue(EndOfStatement, start, start);
                }
                self.if_stack.pop();
            }
            return Ok(token.clone());
        }
        // Ident
        Ok(Ident(ident))
    }

    fn handle_scope_start(&mut self, start: Location) -> AddTokensResult {
        let end = Location::new(start.line, start.col + 1);
        self.source.next(); // consume >
        self.consume_whitespace();
        if self.source.peek() == Some(&'#') {
            self.source.next();
            self.consume_comment();
        }
        if self.source.peek().is_none() {
            return Err(ScanErr::new(ErrKind::ExpectedBlock, self.source.loc()));
        } else if self.next_char_is('\n') {
            // Block
            self.add_token_to_queue(Token::ScopeStart, start, end);
            self.expect_indent()?;
        } else {
            // Inline block
            let end = Location::new(start.line, start.col + 1);
            self.add_token_to_queue(Token::InlineScopeStart, start, end);
            self.inline_scope_stack.push(start);
        }
        Ok(())
    }

    fn handle_newline(&mut self, loc: Location) -> AddTokensResult {
        if self.bracket_stack.size() == 0 {
            self.maybe_exit_inline_scope(loc, false);
            self.maybe_add_end_of_statement_token(loc);
            self.maybe_dedent()?;
        } else {
            self.consume_whitespace();
        }
        Ok(())
    }

    fn handle_end_of_input(&mut self, loc: Location) -> AddTokensResult {
        if let Some((c, bracket_loc)) = self.bracket_stack.pop() {
            return Err(ScanErr::new(ErrKind::UnmatchedOpeningBracket(c), bracket_loc));
        }
        self.add_token_to_queue(Token::EndOfInput, loc, loc);
        Ok(())
    }

    fn maybe_add_end_of_statement_token(&mut self, loc: Location) {
        match self.last_token() {
            Token::EndOfStatement => (),
            _ => self.add_token_to_queue(Token::EndOfStatement, loc, loc),
        }
    }

    // Indentation Handlers --------------------------------------------

    fn assert_start_of_line(&self, name: &str) {
        assert_eq!(
            self.source.current_char,
            Some('\n'),
            "Method should only be called at the start of a line: {name}",
        );
    }

    /// Get the next indent level. Blank lines, whitespace-only lines,
    /// and comment-only lines are skipped over.
    fn get_next_indent_level(&mut self) -> Result<u8, ScanErr> {
        use ErrKind::{InvalidIndent, WhitespaceAfterIndent};
        let next_level = loop {
            let num_spaces = self.consume_contiguous(' ');
            let whitespace_count = self.consume_whitespace();
            if let Some(char) = self.source.peek() {
                if *char == '\n' {
                    // Blank or whitespace-only line; skip it.
                    self.source.next();
                    continue;
                } else if *char == '#' {
                    self.consume_comment();
                    continue;
                }
                if num_spaces % 4 != 0 {
                    let loc = self.source.loc();
                    return Err(ScanErr::new(InvalidIndent(num_spaces), loc));
                }
                if whitespace_count > 0 {
                    let loc = self.source.loc();
                    return Err(ScanErr::new(WhitespaceAfterIndent, loc));
                }
                break num_spaces / 4;
            } else {
                break 0;
            }
        };
        Ok(next_level)
    }

    /// Expect the indent level to increase by one.
    fn expect_indent(&mut self) -> AddTokensResult {
        self.assert_start_of_line("expect_indent");
        let loc = self.source.loc();
        let current_level = self.indent_level;
        let expected_level = current_level + 1;
        let new_level = self.get_next_indent_level()?;
        if new_level < expected_level {
            return Err(ScanErr::new(
                ErrKind::ExpectedIndentedBlock(expected_level),
                loc,
            ));
        }
        self.set_indent_level(new_level, loc)
    }

    /// Handle dedent after a newline is encountered.
    fn maybe_dedent(&mut self) -> AddTokensResult {
        self.assert_start_of_line("dedent");
        let loc = self.source.loc();
        let line = if loc.line == 1 { 1 } else { loc.line + 1 };
        let loc = Location::new(line, 1);
        let next_level = self.get_next_indent_level()?;
        if next_level > self.indent_level {
            return Err(ScanErr::new(ErrKind::UnexpectedIndent(next_level), loc));
        }
        self.set_indent_level(next_level, loc)
    }

    /// Maybe update the current indent level. If the new indent level
    /// is the same as the current indent level, do nothing. If it has
    /// increased, that signals the start of a block (scopes). If it has
    /// decreased, that signals the end of one or more blocks (scopes).
    fn set_indent_level(&mut self, indent_level: u8, loc: Location) -> AddTokensResult {
        let mut current_level = self.indent_level;
        if indent_level == current_level {
            // Stayed the same; nothing to do
        } else if indent_level == current_level + 1 {
            // Increased by one level
            self.indent_level = indent_level;
        } else if indent_level < current_level {
            // Decreased by one or more levels
            while current_level > indent_level {
                self.exit_block_scope(loc);
                current_level -= 1;
            }
            self.indent_level = current_level;
        } else {
            // Increased by *more* than one level
            return Err(ScanErr::new(ErrKind::UnexpectedIndent(indent_level), loc));
        }
        Ok(())
    }

    // Scope handlers --------------------------------------------------

    fn exit_block_scope(&mut self, loc: Location) {
        self.add_token_to_queue(Token::ScopeEnd, loc, loc);
        self.add_token_to_queue(Token::EndOfStatement, loc, loc);
    }

    /// The scope for an inline block ends when one of the following
    /// tokens is encountered: comma, closing bracket, newline, end of
    /// input.
    ///
    /// If exiting because an `else` was encountered, exit back to the
    /// matching `if`. If inside a bracket group, exit only as far back
    /// as the start of the group. Otherwise, all inline scopes are
    /// exited.
    fn exit_inline_scope(&mut self, loc: Location, is_else: bool) -> bool {
        let bracket_loc = match self.bracket_stack.peek() {
            Some((_, bracket_loc)) => (bracket_loc.line, bracket_loc.col),
            None => (0, 0),
        };
        let if_loc = match self.if_stack.peek() {
            Some(if_loc) => (if_loc.line, if_loc.col),
            None => (0, 0),
        };
        let mut count = 0;
        while let Some(scope_start) = self.inline_scope_stack.peek() {
            let scope_loc = (scope_start.line, scope_start.col);
            if scope_loc > bracket_loc {
                if is_else && scope_loc < if_loc {
                    break;
                }
                self.inline_scope_stack.pop();
                self.add_token_to_queue(Token::EndOfStatement, loc, loc);
                self.add_token_to_queue(Token::InlineScopeEnd, loc, loc);
                count += 1;
            } else {
                break;
            }
        }
        count > 0
    }

    fn maybe_exit_inline_scope(&mut self, loc: Location, is_else: bool) -> bool {
        if !self.inline_scope_stack.is_empty() {
            return self.exit_inline_scope(loc, is_else);
        }
        false
    }

    // Utilities -------------------------------------------------------

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

    /// Consume the next character if it's equal to the specified
    /// character.
    fn next_char_is(&mut self, char: char) -> bool {
        match self.source.peek() {
            Some(next_char) => {
                if *next_char == char {
                    self.source.next();
                    true
                } else {
                    false
                }
            }
            None => false,
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

    /// Consume the next character and return the specified token.
    fn consume_char_and_return_token(&mut self, token: Token) -> Token {
        self.source.next();
        token
    }

    /// Consume the next two characters and return the specified token.
    fn consume_two_chars_and_return_token(&mut self, token: Token) -> Token {
        self.source.next();
        self.source.next();
        token
    }

    /// Check the specified closing bracket to ensure the last opening
    /// bracket matches. If it does, the specified token is returned.
    fn pop_bracket_and_return_token(
        &mut self,
        close: char,
        loc: Location,
        token: Token,
    ) -> Result<Token, ScanErr> {
        if let Some((open, _)) = self.bracket_stack.pop() {
            if matches!((open, close), ('(', ')') | ('[', ']') | ('{', '}')) {
                return Ok(token);
            }
        }
        Err(ScanErr::new(ErrKind::UnmatchedClosingBracket(close), loc))
    }

    /// Consume contiguous whitespace up to the end of the line. Return
    /// the number of whitespace characters consumed.
    fn consume_whitespace(&mut self) -> u8 {
        let mut count = 0;
        loop {
            match self.next_char_if(|&c| c != '\n' && c.is_whitespace()) {
                Some(_) => count += 1,
                None => break count,
            }
        }
    }

    /// Consume comment characters up to newline.
    fn consume_comment(&mut self) {
        while self.next_char_if(|&c| c != '\n').is_some() {}
    }

    /// Consume contiguous chars and return count.
    fn consume_contiguous(&mut self, char: char) -> u8 {
        let mut count = 0;
        while self.next_char_is(char) {
            count += 1;
        }
        count
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
            self.source.next();
        }
        string.push_str(self.collect_digits(radix).as_str());
        if radix == 10 {
            if let Some((dot, digit, _)) =
                self.next_two_chars_if(|&c| c == '.', |&d| d.is_digit(radix))
            {
                // If the number is followed by a dot and at least one
                // digit consume the dot, the digit, and any following
                // digits.
                string.push(dot);
                string.push(digit);
                string.push_str(self.collect_digits(radix).as_str());
            }
            // Handle E notation *without* sign.
            if let Some((_, digit, _)) = self
                .next_two_chars_if(|&c| c == 'e' || c == 'E', |&e| e.is_digit(radix))
            {
                string.push('E');
                string.push('+');
                string.push(digit);
                string.push_str(self.collect_digits(radix).as_str());
            }
            // Handle E notation *with* sign.
            if let Some((_, sign, digit)) = self.next_three_chars_if(
                |&c| c == 'e' || c == 'E',
                |&d| d == '+' || d == '-',
                |&e| e.is_digit(radix),
            ) {
                string.push('E');
                string.push(sign);
                string.push(digit);
                string.push_str(self.collect_digits(radix).as_str());
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

                    'a' => string.push('\x07'), // bell
                    'b' => string.push('\x08'), // backspace
                    'f' => string.push('\x0c'), // form feed

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

                    // Unescape escaped single quote. Seems to be
                    // standard (Python and Rust both do it).
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
                match self.source.next() {
                    // Found closing quote; return string.
                    Some(c) if c == quote => break (string, true),
                    // Append current char and continue.
                    Some(c) => string.push(c),
                    // End of input reached without finding closing quote :(
                    None => {
                        if self.source.newline_added {
                            string.pop();
                        }
                        break (string, false);
                    }
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
    fn read_ident(&mut self, first_char: char) -> String {
        let mut string = first_char.to_string();
        loop {
            match self.next_char_if(|&c| {
                c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'
            }) {
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
            match self.next_char_if(|&c| c.is_ascii_alphabetic() || c.is_ascii_digit())
            {
                Some((c, _, _)) => string.push(c),
                None => break string,
            }
        }
    }
}

impl<'a, T: BufRead> Iterator for Scanner<'a, T> {
    type Item = ScanTokenResult;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token_from_queue() {
            Ok(TokenWithLocation { token: Token::EndOfInput, .. }) => None,
            Ok(token) => Some(Ok(token)),
            err => Some(err),
        }
    }
}
