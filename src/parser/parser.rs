//! Parse a stream of tokens into an AST.
use std::collections::VecDeque;
use std::iter::{Iterator, Peekable};

use crate::ast;
use crate::format::FormatStringToken;
use crate::parser::result::StatementResult;
use crate::scanner::{ScanErr, ScanResult, Token, TokenWithLocation};
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
    let scanner: Vec<ScanResult> = vec![];
    let mut parser = Parser::new(scanner.into_iter());
    parser.lookahead_queue.extend(tokens);
    parser.parse()
}

pub struct Parser<I: Iterator<Item = ScanResult>> {
    current_token: Option<TokenWithLocation>,
    token_stream: Peekable<I>,
    lookahead_queue: VecDeque<TokenWithLocation>,
}

impl<I: Iterator<Item = ScanResult>> Parser<I> {
    pub fn new(token_iter: I) -> Self {
        Self {
            current_token: None,
            token_stream: token_iter.peekable(),
            lookahead_queue: VecDeque::new(),
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
                self.err(ParseErrKind::ExpectedToken(self.next_loc(), token.clone()))
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
        assert!(tokens.len() > 1, "At least two tokens are required");
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

    // Utilities -------------------------------------------------------

    /// Make creating errors a little less tedious.
    fn err(&self, kind: ParseErrKind) -> ParseErr {
        ParseErr::new(kind)
    }

    fn scan_err(&self, err: ScanErr) -> ParseErr {
        self.err(ParseErrKind::ScanErr(err))
    }

    /// Collect tokens until the specified token is reached. This is
    /// used for lookahead. For example, it's used to find the
    /// parameters/args for a function def/call since the number of
    /// args is unknown up front and we can't use single-peek token
    /// inspection techniques.
    fn collect_until(
        &mut self,
        token: &Token,
    ) -> Result<(bool, Vec<TokenWithLocation>), ParseErr> {
        let mut collector = vec![];
        let mut nesting_stack = vec![];
        while let Some(t) = self.next_token()? {
            if &t.token == token && nesting_stack.is_empty() {
                return Ok((true, collector));
            }
            match t.token {
                Token::LParen => nesting_stack.push('('),
                Token::LBracket => nesting_stack.push('['),
                Token::RParen => {
                    if nesting_stack.pop() != Some('(') {
                        return Err(
                            self.err(ParseErrKind::MismatchedBracket(self.loc()))
                        );
                    }
                }
                Token::RBracket => {
                    if nesting_stack.pop() != Some('[') {
                        return Err(
                            self.err(ParseErrKind::MismatchedBracket(self.loc()))
                        );
                    }
                }
                _ => (),
            }
            collector.push(t);
        }
        Ok((false, collector))
    }

    /// Expect the start of a scope. This is really just a check to
    /// make sure the token stream is valid.
    fn expect_scope(&mut self) -> Result<(), ParseErr> {
        self.expect_token(&Token::ScopeStart)?;
        Ok(())
    }

    /// Expect the end of a scope.
    fn expect_scope_end(&mut self) -> Result<(), ParseErr> {
        self.expect_token(&Token::ScopeEnd)?;
        self.expect_token(&Token::EndOfStatement)?;
        Ok(())
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
        let statement = match token.token {
            // XXX: The print statement is temporary until functions
            //      are implemented. The shenanigans below are so that
            //      print statements have similar syntax to the eventual
            //      built in print function.
            Print => {
                self.expect_token(&LParen)?;
                let expr = if self.peek_token_is(&RParen)? {
                    ast::Expr::new_string("")
                } else if self.has_tokens()? {
                    self.expr(0)?
                } else {
                    return Err(self.err(ParseErrKind::ExpectedExpr(self.next_loc())));
                };
                self.expect_token(&RParen)?;
                self.expect_token(&EndOfStatement)?;
                ast::Statement::new_print(expr)
            }
            Jump => {
                if let Some(ident_token) = self.next_token()? {
                    if let Ident(name) = ident_token.token {
                        self.expect_token(&EndOfStatement)?;
                        ast::Statement::new_jump(name)
                    } else {
                        return Err(
                            self.err(ParseErrKind::UnexpectedToken(ident_token))
                        );
                    }
                } else {
                    return Err(self.err(ParseErrKind::ExpectedIdent(self.next_loc())));
                }
            }
            Label(name) => ast::Statement::new_label(name),
            _ => {
                self.lookahead_queue.push_front(token);
                let expr = self.expr(0)?;
                ast::Statement::new_expr(expr)
            }
        };
        // Consume optional EOS
        self.next_token_is(&EndOfStatement)?;
        Ok(statement)
    }

    /// Get the next expression, possibly recurring to handle nested
    /// expressions, unary & binary expressions, blocks, functions, etc.
    fn expr(&mut self, prec: u8) -> ExprResult {
        use Token::*;
        let token = self.expect_next_token()?;
        let mut expr = match token.token {
            LParen => match self.next_token_is(&RParen)? {
                true => ast::Expr::new_tuple(vec![]),
                false => self.nested_expr()?,
            },
            Nil => ast::Expr::new_literal(ast::Literal::new_nil()),
            True => ast::Expr::new_literal(ast::Literal::new_bool(true)),
            False => ast::Expr::new_literal(ast::Literal::new_bool(false)),
            Float(value) => ast::Expr::new_literal(ast::Literal::new_float(value)),
            Int(value) => ast::Expr::new_literal(ast::Literal::new_int(value)),
            String(value) => ast::Expr::new_literal(ast::Literal::new_string(value)),
            FormatString(tokens) => self.format_string(tokens)?,
            Ident(name) => {
                if self.next_token_is(&LParen)? {
                    // Function def or call
                    return Ok(self.func(name)?);
                }
                ast::Expr::new_ident(ast::Ident::new_ident(name))
            }
            Block => {
                let block = self.block()?;
                ast::Expr::new_block(block)
            }
            If => {
                let mut branches = vec![];
                branches.push((self.expr(0)?, self.block()?));
                loop {
                    match self.next_tokens_are(vec![&Else, &If])? {
                        true => branches.push((self.expr(0)?, self.block()?)),
                        false => break,
                    }
                }
                let default = match self.next_token_is(&Else)? {
                    true => Some(self.block()?),
                    false => None,
                };
                ast::Expr::new_conditional(branches, default)
            }
            Loop => {
                let expr = match self.peek_token_is(&ScopeStart)? {
                    true => ast::Expr::new_literal(ast::Literal::new_bool(true)),
                    false => self.expr(0)?,
                };
                let block = self.block()?;
                ast::Expr::new_loop(expr, block)
            }
            Break => ast::Expr::new_break(match self.peek_token()? {
                Some(TokenWithLocation { token: EndOfStatement, .. }) | None => {
                    ast::Expr::new_literal(ast::Literal::new_nil())
                }
                _ => self.expr(0)?,
            }),
            _ => self.expect_unary_expr(&token)?,
        };
        expr = if self.next_token_is(&Comma)? {
            self.tuple(expr)?
        } else {
            self.maybe_binary_expr(prec, expr)?
        };
        Ok(expr)
    }

    fn format_string(
        &mut self,
        format_string_tokens: Vec<FormatStringToken>,
    ) -> ExprResult {
        let mut items = vec![];
        for format_string_token in format_string_tokens {
            match format_string_token {
                FormatStringToken::Str(value) => {
                    items.push(ast::Expr::new_literal(ast::Literal::new_string(value)));
                }
                FormatStringToken::Expr(tokens) => {
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
        Ok(ast::Expr::new_format_string(items))
    }

    fn tuple(&mut self, first_expr: ast::Expr) -> ExprResult {
        let mut items = vec![first_expr];
        loop {
            if !self.has_tokens()?
                || self.peek_token_is(&Token::RParen)?
                || self.next_token_is(&Token::EndOfStatement)?
            {
                break;
            }
            if self.next_token_is(&Token::LParen)? {
                let expr = self.nested_expr()?;
                items.push(expr);
            } else {
                let expr = self.expr(0)?;
                if let ast::Expr { kind: ast::ExprKind::Tuple(new_items) } = expr {
                    items.extend(new_items);
                } else {
                    items.push(expr);
                }
            };
        }
        Ok(ast::Expr::new_tuple(items))
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
        Ok(ast::Expr::new_unary_op(op, rhs))
    }

    /// See if the expr is followed by an infix operator. If so, get the
    /// RHS expression and return a binary expression. If not, just
    /// return the original expr.
    fn maybe_binary_expr(&mut self, prec: u8, mut expr: ast::Expr) -> ExprResult {
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
                expr = ast::Expr::new_binary_op(expr, op, rhs);
            } else {
                break Ok(expr);
            }
        }
    }

    /// Handle nested expressions (inside parens).
    fn nested_expr(&mut self) -> ExprResult {
        if !self.has_tokens()? {
            return Err(self.err(ParseErrKind::ExpectedExpr(self.next_loc())));
        }
        let expr = self.expr(0)?;
        if self.next_token_is(&Token::RParen)? {
            return Ok(expr);
        }
        self.nested_expr()
    }

    /// Handle `block ->`, `if <expr> ->`, etc.
    fn block(&mut self) -> BlockResult {
        self.expect_scope()?;
        let statements = self.statements()?;
        if statements.is_empty() {
            return Err(self.err(ParseErrKind::ExpectedBlock(self.next_loc())));
        }
        self.expect_scope_end()?;
        Ok(ast::Block::new(statements))
    }

    /// Handle `func () -> ...` (definition) and `func()` (call).
    fn func(&mut self, name: String) -> ExprResult {
        let (found, mut tokens) = self.collect_until(&Token::RParen)?;

        if !found {
            self.lookahead_queue.extend(tokens);
            return Err(
                self.err(ParseErrKind::ExpectedToken(self.next_loc(), Token::RParen))
            );
        }

        // Add a trailing comma for consistency
        if let Some(t) = tokens.last() {
            if t.token != Token::Comma {
                let start = Location::new(t.start.line, t.start.col + 1);
                tokens.push(TokenWithLocation::new(Token::Comma, start, start));
            }
        }

        if self.peek_token_is(&Token::ScopeStart)? {
            // Function def - tokens are parameters
            let params = self.parse_params(tokens)?;
            let block = self.block()?;
            Ok(ast::Expr::new_func(name.clone(), params, block))
        } else {
            // Function call -- tokens are args
            let args = parse_tokens(tokens)?;
            let args = args.statements;
            let args = vec![];
            Ok(ast::Expr::new_call(name.clone(), args))
        }
    }

    fn parse_params(
        &self,
        tokens: Vec<TokenWithLocation>,
    ) -> Result<Vec<String>, ParseErr> {
        let mut params = vec![];

        if tokens.is_empty() {
            return Ok(params);
        }

        let start = tokens[0].start.clone();
        let program = parse_tokens(tokens)?;
        let statements = program.statements;

        if statements.is_empty() || statements.len() > 1 {
            return Err(self.err(ParseErrKind::SyntaxErr(start)));
        }

        let statement = statements.last().unwrap();

        if let Some(items) = statement.tuple_items() {
            for expr in items.iter() {
                if let Some(name) = expr.ident_name() {
                    params.push(name.clone());
                } else {
                    return Err(self.err(ParseErrKind::SyntaxErr(start)));
                }
            }
        } else {
            return Err(self.err(ParseErrKind::SyntaxErr(start)));
        }

        Ok(params)
    }
}
