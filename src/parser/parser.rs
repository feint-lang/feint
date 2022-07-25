//! Parse a stream of tokens into an AST.
use std::collections::VecDeque;
use std::iter::{Iterator, Peekable};

use crate::ast;
use crate::format::FormatStrToken;
use crate::parser::result::StatementResult;
use crate::scanner::{ScanErr, ScanTokenResult, Token, TokenWithLocation};
use crate::util::Location;

use super::precedence::{
    get_binary_precedence, get_unary_precedence, is_right_associative,
};
use super::result::{
    BlockResult, BoolResult, ExprResult, NextInfixResult, NextTokenResult, ParseErr,
    ParseErrKind, ParseResult, PeekTokenResult, StatementsResult,
};

/// Parse tokens and return the resulting AST or error.
pub fn parse_tokens(tokens: Vec<TokenWithLocation>) -> ParseResult {
    let scanner: Vec<ScanTokenResult> = vec![];
    let mut parser = Parser::new(scanner.into_iter());
    parser.lookahead_queue.extend(tokens);
    parser.parse()
}

pub struct Parser<I: Iterator<Item = ScanTokenResult>> {
    current_token: Option<TokenWithLocation>,
    token_stream: Peekable<I>,
    lookahead_queue: VecDeque<TokenWithLocation>,
    loop_level: u8,
}

impl<I: Iterator<Item = ScanTokenResult>> Parser<I> {
    pub fn new(token_iter: I) -> Self {
        Self {
            current_token: None,
            token_stream: token_iter.peekable(),
            lookahead_queue: VecDeque::new(),
            loop_level: 0,
        }
    }

    // Parse entry point -----------------------------------------------

    /// A program is a list of statements.
    pub fn parse(&mut self) -> ParseResult {
        let statements = self.statements()?;
        let program = ast::Program::new(statements);
        Ok(program)
    }

    // Tokens ----------------------------------------------------------

    /// Get the location of the current token.
    fn loc(&self) -> Location {
        match &self.current_token {
            Some(t) => t.start,
            None => Location::new(0, 0),
        }
    }

    /// Get location after current token.
    fn next_loc(&self) -> Location {
        if let Some(t) = &self.current_token {
            match t.token {
                Token::EndOfStatement => Location::new(t.end.line + 1, 1),
                _ => Location::new(t.end.line, t.end.col + 1),
            }
        } else {
            Location::new(0, 0)
        }
    }

    /// Are there any tokens left in the stream?
    fn has_tokens(&mut self) -> BoolResult {
        Ok(self.peek_token()?.is_some())
    }

    /// Consume and return the next token unconditionally. If no tokens
    /// are left, return `None`.
    fn next_token(&mut self) -> NextTokenResult {
        if let Some(t) = self.lookahead_queue.pop_front() {
            self.current_token = Some(t.clone());
            return Ok(Some(t));
        }
        if let Some(result) = self.token_stream.next() {
            return result
                .map(|t| {
                    self.current_token = Some(t.clone());
                    Some(t)
                })
                .map_err(|err| self.scan_err(err.clone()));
        }
        Ok(None)
    }

    /// Get the next token. If there isn't a next token, panic! This is
    /// used where there *should* be a next token and if there isn't
    /// that indicates an internal logic/processing error.
    fn expect_next_token(&mut self) -> Result<TokenWithLocation, ParseErr> {
        Ok(self.next_token()?.expect("Expected token"))
    }

    /// Expect the next token to be the specified token. If it is,
    /// consume the token and return nothing. If it's not, return an
    /// error.
    fn expect_token(&mut self, token: &Token) -> Result<(), ParseErr> {
        if !self.next_token_is(token)? {
            return Err(
                self.err(ParseErrKind::ExpectedToken(self.loc(), token.clone()))
            );
        }
        Ok(())
    }

    /// Consume the next token and return it if the specified condition
    /// is true. Otherwise, return `None`.
    fn next_token_if(&mut self, func: impl FnOnce(&Token) -> bool) -> NextTokenResult {
        if let Some(t) = self.peek_token()? {
            if func(&t.token) {
                return Ok(self.next_token()?);
            }
        }
        Ok(None)
    }

    /// Consume next token and return true *if* the next token is equal
    /// to specified token. Otherwise, leave the token in the stream and
    /// return false.
    fn next_token_is(&mut self, token: &Token) -> BoolResult {
        if let Some(_) = self.next_token_if(|t| t == token)? {
            return Ok(true);
        }
        Ok(false)
    }

    /// Consume next N tokens and return true *if* the next N tokens are
    /// equal to specified tokens. Otherwise, leave the tokens in the
    /// stream and return false.
    fn next_tokens_are(&mut self, tokens: Vec<&Token>) -> BoolResult {
        assert!(tokens.len() > 0, "At least one token is required");
        let mut temp_queue: VecDeque<TokenWithLocation> = VecDeque::new();
        for token in tokens {
            match self.next_token_if(|t| t == token)? {
                Some(token) => {
                    temp_queue.push_front(token);
                }
                None => {
                    for twl in temp_queue {
                        self.lookahead_queue.push_front(twl);
                    }
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// Return the next token along with its precedence *if* it's both
    /// an infix operator *and* its precedence is greater than the
    /// current precedence level.
    fn next_infix_token(&mut self, current_prec: u8) -> NextInfixResult {
        if let Some(token) = self.next_token_if(|t| {
            let p = get_binary_precedence(t);
            p > 0 && p > current_prec
        })? {
            let prec = get_binary_precedence(&token.token);
            return Ok(Some((token, prec)));
        }
        Ok(None)
    }

    /// Return the next token without consuming it. If no tokens are
    /// left, return `None`.
    fn peek_token(&mut self) -> PeekTokenResult {
        if let Some(t) = self.lookahead_queue.front() {
            return Ok(Some(t));
        }
        if let Some(result) = self.token_stream.peek() {
            return result
                .as_ref()
                .map(|t| Some(t))
                .map_err(|err| ParseErr::new(ParseErrKind::ScanErr(err.clone())));
        }
        Ok(None)
    }

    /// Look at the next token and return it if it's equal to the
    /// specified token. Otherwise, return `None`.
    fn peek_token_if(&mut self, func: impl FnOnce(&Token) -> bool) -> PeekTokenResult {
        if let Some(t) = self.peek_token()? {
            if func(&t.token) {
                return Ok(Some(t));
            }
        }
        Ok(None)
    }

    /// Look at the next token and return true if it's equal to the
    /// specified token. Otherwise, return false.
    fn peek_token_is(&mut self, token: &Token) -> BoolResult {
        if let Some(_) = self.peek_token_if(|t| t == token)? {
            return Ok(true);
        }
        Ok(false)
    }

    /// Check whether the next token is a block or inline scope start
    /// token.
    fn peek_token_is_scope_start(&mut self) -> BoolResult {
        use Token::{InlineScopeStart, ScopeStart};
        if let Some(TokenWithLocation { token, .. }) = self.peek_token()? {
            Ok(token == &ScopeStart || token == &InlineScopeStart)
        } else {
            Ok(false)
        }
    }

    // Utilities -------------------------------------------------------

    /// Make creating errors a little less tedious.
    fn err(&self, kind: ParseErrKind) -> ParseErr {
        ParseErr::new(kind)
    }

    fn scan_err(&self, err: ScanErr) -> ParseErr {
        self.err(ParseErrKind::ScanErr(err))
    }

    // Grammar ---------------------------------------------------------

    /// Get a list of statements. Collect statements until there's
    /// either no more input or the end of a scope is reached.
    fn statements(&mut self) -> StatementsResult {
        let mut statements = vec![];
        loop {
            if !self.has_tokens()? || self.peek_token_is(&Token::ScopeEnd)? {
                break;
            }
            let statement = self.statement()?;
            statements.push(statement);
        }
        Ok(statements)
    }

    /// Get the next statement (which might be an expression). In
    /// certain cases, multiple statements may be returned (e.g.,
    /// loops).
    fn statement(&mut self) -> StatementResult {
        use Token::*;
        let token = self.expect_next_token()?;
        let start = token.start;
        let statement = match token.token {
            Jump => self.jump(start)?,
            Label(name) => {
                let label = ast::Statement::new_label(name, start, token.end);
                if !self.peek_token_is(&EndOfStatement)? {
                    return Ok(label);
                }
                label
            }
            Continue => self.continue_(start, token.end)?,
            _ => {
                self.lookahead_queue.push_front(token);
                let expr = self.expr(0)?;
                let end = expr.end;
                ast::Statement::new_expr(expr, start, end)
            }
        };
        self.expect_token(&EndOfStatement)?;
        Ok(statement)
    }

    /// Handle jump statement.
    fn jump(&mut self, start: Location) -> StatementResult {
        if let Some(ident_token) = self.next_token()? {
            if let Token::Ident(name) = ident_token.token {
                Ok(ast::Statement::new_jump(name, start, ident_token.end))
            } else {
                return Err(self.err(ParseErrKind::UnexpectedToken(ident_token)));
            }
        } else {
            return Err(self.err(ParseErrKind::ExpectedIdent(self.next_loc())));
        }
    }

    /// Get the next expression, possibly recurring to handle nested
    /// expressions, unary & binary expressions, blocks, functions, etc.
    fn expr(&mut self, prec: u8) -> ExprResult {
        use Token::*;
        let token = self.expect_next_token()?;
        let start = token.start;
        let end = token.end; // Default end location for simple expressions
        let expr = match token.token {
            LParen => self.parenthesized(start)?,
            Nil => ast::Expr::new_nil(start, end),
            True => ast::Expr::new_true(start, end),
            False => ast::Expr::new_false(start, end),
            Int(value) => ast::Expr::new_int(value, start, end),
            Float(value) => ast::Expr::new_float(value, start, end),
            Str(string) => ast::Expr::new_string(string, start, end),
            FormatStr(tokens) => self.format_string(tokens)?,
            Block => {
                let block = self.block()?;
                let end = block.end;
                ast::Expr::new_block(block, start, end)
            }
            If => self.conditional(start)?,
            Loop => self.loop_(start)?,
            Break => self.break_(start)?,
            Ident(name) => match self.next_token_is(&LParen)? {
                true => self.func(name, start)?,
                false => ast::Expr::new_ident(ast::Ident::new_ident(name), start, end),
            },
            _ => self.expect_unary_expr(&token)?,
        };
        // If the expression is followed by a binary operator, a binary
        // expression will be parsed and returned. Otherwise, the
        // expression will be returned as is.
        let expr = self.maybe_binary_expr(prec, expr)?;
        Ok(expr)
    }

    /// Handle parenthesized expressions. There are two cases:
    ///
    /// 1. A grouped expression such as `(1)` or `(1 + 2)`
    /// 2. A tuple such as `(1,)` or `(1, 2)`
    fn parenthesized(&mut self, start: Location) -> ExprResult {
        use Token::{Comma, RParen};
        let expr = match self.next_token_is(&RParen)? {
            true => {
                // () is parsed as a tuple with 0 items
                ast::Expr::new_tuple(vec![], start, self.loc())
            }
            false => {
                let first_item = self.expr(0)?;
                if self.peek_token_is(&Comma)? {
                    let mut items = vec![];
                    items.push(first_item);
                    loop {
                        if self.next_token_is(&RParen)? {
                            break;
                        }
                        self.expect_token(&Comma)?;
                        if self.next_token_is(&RParen)? {
                            break;
                        }
                        let item = self.expr(0)?;
                        items.push(item);
                    }
                    ast::Expr::new_tuple(items, start, self.loc())
                } else {
                    self.expect_token(&RParen)?;
                    first_item
                }
            }
        };
        Ok(expr)
    }

    /// Handle format strings (AKA $ strings).
    fn format_string(
        &mut self,
        format_string_tokens: Vec<FormatStrToken>,
    ) -> ExprResult {
        let start = self.loc();
        let mut items = vec![];
        for format_string_token in format_string_tokens {
            match format_string_token {
                FormatStrToken::Str(value) => {
                    // TODO: Fix location
                    items.push(ast::Expr::new_string(
                        value,
                        Location::new(1, 1),
                        Location::new(1, 1),
                    ));
                }
                FormatStrToken::Expr(tokens) => {
                    // TODO: Make location more precise
                    let loc = self.current_token.as_ref().unwrap().start;
                    let program = parse_tokens(tokens)?;
                    for statement in program.statements {
                        match statement.kind {
                            ast::StatementKind::Expr(expr) => items.push(expr),
                            _ => {
                                return Err(self.err(ParseErrKind::ExpectedExpr(loc)));
                            }
                        }
                    }
                }
            };
        }
        // TODO: Fix end
        Ok(ast::Expr::new_format_string(items, start, self.next_loc()))
    }

    /// Handle `block ->`, `if <expr> ->`, etc.
    fn block(&mut self) -> BlockResult {
        use ParseErrKind::{ExpectedBlock, ExpectedToken};
        use Token::{InlineScopeEnd, InlineScopeStart, ScopeEnd, ScopeStart};
        let statements = if self.next_token_is(&ScopeStart)? {
            let statements = self.statements()?;
            if statements.is_empty() {
                return Err(self.err(ExpectedBlock(self.next_loc())));
            }
            self.expect_token(&ScopeEnd)?;
            statements
        } else if self.next_token_is(&InlineScopeStart)? {
            let statement = self.statement()?;
            self.expect_token(&InlineScopeEnd)?;
            vec![statement]
        } else {
            return Err(self.err(ExpectedToken(self.next_loc(), ScopeStart)));
        };
        let start = statements[0].start;
        let end = statements[statements.len() - 1].end;
        Ok(ast::Block::new(statements, start, end))
    }

    /// Handle `if <expr> -> ...`.
    fn conditional(&mut self, start: Location) -> ExprResult {
        use Token::{Else, EndOfStatement, If};
        let mut branches = vec![];
        let mut end;
        let cond = self.expr(0)?;
        let block = self.block()?;
        end = block.end;
        branches.push((cond, block));
        loop {
            match self.next_tokens_are(vec![&EndOfStatement, &Else, &If])? {
                true => {
                    let cond = self.expr(0)?;
                    let block = self.block()?;
                    end = block.end;
                    branches.push((cond, block))
                }
                false => break,
            }
        }
        let default = match self.next_tokens_are(vec![&EndOfStatement, &Else])? {
            true => {
                let block = self.block()?;
                end = block.end;
                Some(block)
            }
            false => None,
        };
        Ok(ast::Expr::new_conditional(branches, default, start, end))
    }

    /// Handle `loop -> ...` and `loop <cond> -> ...` (`while` loops).
    /// TODO: Handle `for` loops.
    fn loop_(&mut self, start: Location) -> ExprResult {
        self.loop_level += 1;
        let cond = match self.peek_token_is_scope_start()? {
            true => ast::Expr::new_true(self.next_loc(), self.next_loc()),
            false => self.expr(0)?,
        };
        let block = self.block()?;
        let end = block.end;
        self.loop_level -= 1;
        Ok(ast::Expr::new_loop(cond, block, start, end))
    }

    /// Handle `break`, ensuring it's contained in a `loop`.
    fn break_(&mut self, start: Location) -> ExprResult {
        if self.loop_level == 0 {
            return Err(self.err(ParseErrKind::UnexpectedBreak(start)));
        }
        let expr = match self.peek_token()? {
            Some(TokenWithLocation { token: Token::EndOfStatement, .. }) | None => {
                ast::Expr::new_nil(start, start)
            }
            _ => self.expr(0)?,
        };
        let end = expr.end;
        Ok(ast::Expr::new_break(expr, start, end))
    }

    /// Handle `continue`, ensuring it's contained in a `loop`.
    fn continue_(&mut self, start: Location, end: Location) -> StatementResult {
        if self.loop_level == 0 {
            return Err(self.err(ParseErrKind::UnexpectedContinue(start)));
        }
        Ok(ast::Statement::new_continue(start, end))
    }

    /// Handle `func () -> ...` (definition) and `func()` (call).
    fn func(&mut self, name: String, start: Location) -> ExprResult {
        let expr = self.parenthesized(self.loc())?;
        let call_end = expr.end;
        let items = match expr.kind {
            ast::ExprKind::Tuple(items) => items,
            _ => vec![expr],
        };
        if self.peek_token_is_scope_start()? {
            // Function definition
            let mut params = vec![];
            // Ensure all items are identifiers
            for item in items.iter() {
                match &item.kind {
                    ast::ExprKind::Ident(ast::Ident {
                        kind: ast::IdentKind::Ident(name),
                    }) => params.push(name.clone()),
                    _ => return Err(self.err(ParseErrKind::ExpectedIdent(item.start))),
                }
            }
            let block = self.block()?;
            let def_end = block.end;
            Ok(ast::Expr::new_func(name.clone(), params, block, start, def_end))
        } else {
            // Function call
            Ok(ast::Expr::new_call(name.clone(), items, start, call_end))
        }
    }

    /// The current token should represent a unary operator and should
    /// be followed by an expression.
    fn expect_unary_expr(&mut self, op_token: &TokenWithLocation) -> ExprResult {
        let prec = get_unary_precedence(&op_token.token);
        if prec == 0 {
            return Err(self.err(ParseErrKind::UnexpectedToken(op_token.clone())));
        }
        if !self.has_tokens()? {
            return Err(self.err(ParseErrKind::ExpectedOperand(op_token.end)));
        }
        let rhs = self.expr(prec)?;
        let op = op_token.as_str();
        let (start, end) = (op_token.start, rhs.end);
        Ok(ast::Expr::new_unary_op(op, rhs, start, end))
    }

    /// See if the expr is followed by an infix operator. If so, get the
    /// RHS expression and return a binary expression. If not, just
    /// return the original expr.
    fn maybe_binary_expr(&mut self, prec: u8, mut expr: ast::Expr) -> ExprResult {
        let start = expr.start;
        loop {
            let next = self.next_infix_token(prec)?;
            if let Some((infix_token, mut infix_prec)) = next {
                if !self.has_tokens()? {
                    return Err(
                        self.err(ParseErrKind::ExpectedOperand(infix_token.end))
                    );
                }
                // Lower precedence of right-associative operator when
                // fetching its RHS expr.
                if is_right_associative(&infix_token.token) {
                    infix_prec -= 1;
                }
                let rhs = self.expr(infix_prec)?;
                let op = infix_token.as_str();
                let end = rhs.end;
                expr = ast::Expr::new_binary_op(expr, op, rhs, start, end);
            } else {
                break Ok(expr);
            }
        }
    }
}
