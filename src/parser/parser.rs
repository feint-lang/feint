//! Parse a stream of tokens into an AST.
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Cursor};
use std::iter::{Iterator, Peekable};

use crate::ast;
use crate::scanner::{ScanResult, Scanner, Token, TokenWithLocation};
use crate::util::Location;

use super::precedence::{
    get_binary_precedence, get_unary_precedence, is_right_associative,
};
use super::result::{
    BoolResult, ExprResult, NextInfixResult, NextTokenResult, ParseErr, ParseErrKind,
    ParseResult, PeekTokenResult, StatementResult, StatementsResult,
};
use crate::ast::ExprKind;
use crate::ast::StatementKind::Expr;

/// Scan the text into tokens, parse the tokens, and return the
/// resulting AST or error.
pub fn parse_text(text: &str, debug: bool) -> ParseResult {
    let scanner = Scanner::<Cursor<&str>>::from_text(text);
    let mut parser = Parser::new(scanner.into_iter());
    handle_result(parser.parse(), debug)
}

/// Scan the file into tokens, parse the tokens, and return the
/// resulting AST or error.
pub fn parse_file(file_path: &str, debug: bool) -> ParseResult {
    let result = Scanner::<BufReader<File>>::from_file(file_path);
    let scanner = match result {
        Ok(scanner) => scanner,
        Err(err) => {
            return Err(ParseErr::new(ParseErrKind::CouldNotOpenSourceFile(
                file_path.to_string(),
                err.to_string(),
            )));
        }
    };
    let mut parser = Parser::new(scanner.into_iter());
    handle_result(parser.parse(), debug)
}

/// Scan text from stdin into tokens, parse the tokens, and return the
/// resulting AST or error.
pub fn parse_stdin(debug: bool) -> ParseResult {
    let scanner = Scanner::<BufReader<io::Stdin>>::from_stdin();
    let mut parser = Parser::new(scanner.into_iter());
    handle_result(parser.parse(), debug)
}

/// Parse tokens and return the resulting AST or error.
pub fn parse_tokens(tokens: Vec<TokenWithLocation>, debug: bool) -> ParseResult {
    let scanner: Vec<ScanResult> = vec![];
    let mut parser = Parser::new(scanner.into_iter());
    parser.lookahead_queue.extend(tokens);
    handle_result(parser.parse(), debug)
}

fn handle_result(result: ParseResult, debug: bool) -> ParseResult {
    result.map(|program| {
        if debug {
            eprintln!("{:=<72}", "AST ");
            eprintln!("{:?}", program);
        };
        program
    })
}

struct Parser<I: Iterator<Item = ScanResult>> {
    current_token: Option<TokenWithLocation>,
    token_stream: Peekable<I>,
    lookahead_queue: VecDeque<TokenWithLocation>,
}

impl<I: Iterator<Item = ScanResult>> Parser<I> {
    fn new(token_iter: I) -> Self {
        Self {
            current_token: None,
            token_stream: token_iter.peekable(),
            lookahead_queue: VecDeque::new(),
        }
    }

    // Parse entry point -----------------------------------------------

    /// A program is a list of statements.
    fn parse(&mut self) -> ParseResult {
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
        match self.token_stream.next() {
            Some(Ok(t)) => {
                self.current_token = Some(t.clone());
                Ok(Some(t))
            }
            Some(Err(err)) => Err(ParseErr::new(ParseErrKind::ScanError(err.clone()))),
            None => Ok(None),
        }
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
            return Err(ParseErr::new(ParseErrKind::SyntaxError(
                format!("Expected {}", token),
                self.next_loc(),
            )));
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
        } else if let Some(result) = self.token_stream.peek() {
            return result
                .as_ref()
                .map(|t| Some(t))
                .map_err(|err| ParseErr::new(ParseErrKind::ScanError(err.clone())));
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

    /// Collect tokens until the specified token is reached. This is
    /// used for lookahead. For example, the is used to find the
    /// parameters/args for a function def/call since the number of
    /// args is unknown up front and we can't use single-peek token
    /// inspection techniques.
    fn collect_until(
        &mut self,
        token: Token,
    ) -> Result<(bool, Vec<TokenWithLocation>), ParseErr> {
        let mut collector = vec![];
        let mut nesting_level = 0;
        while let Some(t) = self.next_token()? {
            if t.token == token && nesting_level == 0 {
                return Ok((true, collector));
            }
            if token == Token::RightParen && t.token == Token::LeftParen {
                nesting_level += 1;
            }
            collector.push(t);
        }
        Ok((false, collector))
    }

    /// Expect the start of a scope. This is really just a check to
    /// make sure the token stream is valid.
    fn expect_scope(&mut self) -> Result<(), ParseErr> {
        self.expect_token(&Token::EndOfStatement)?;
        self.expect_token(&Token::ScopeStart)?;
        Ok(())
    }

    /// Expect and collect a block of statements. There must be at least
    /// one statement.
    fn expect_statement_block(&mut self) -> StatementsResult {
        let statements = self.statements()?;
        if statements.is_empty() {
            return Err(ParseErr::new(ParseErrKind::ExpectedBlock(self.next_loc())));
        }
        self.expect_token(&Token::ScopeEnd)?;
        Ok(statements)
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

    /// Get the next statement (which might be an expression).
    fn statement(&mut self) -> StatementResult {
        let token = self.expect_next_token()?;
        let statement = match token.token {
            /// XXX: The print statement is temporary until functions
            ///      are implemented. The shenanigans below are so that
            ///      print statements have similar syntax to the
            ///      eventual built in print function.
            Token::Print => {
                self.expect_token(&Token::LeftParen)?;
                let expr = if self.peek_token_is(&Token::RightParen)? {
                    ast::Expr::new_string("")
                } else if self.has_tokens()? {
                    self.expr(0)?
                } else {
                    return Err(ParseErr::new(ParseErrKind::ExpectedExpr(
                        self.next_loc(),
                    )));
                };
                self.expect_token(&Token::RightParen)?;
                self.expect_token(&Token::EndOfStatement)?;
                ast::Statement::new_print(expr)
            }
            Token::Jump => {
                if let Some(ident_token) = self.next_token()? {
                    if let Token::Ident(name) = ident_token.token {
                        self.expect_token(&Token::EndOfStatement)?;
                        ast::Statement::new_jump(name)
                    } else {
                        let kind = ParseErrKind::UnexpectedToken(ident_token);
                        return Err(ParseErr::new(kind));
                    }
                } else {
                    let kind = ParseErrKind::ExpectedIdent(token);
                    return Err(ParseErr::new(kind));
                }
            }
            Token::Label(name) => ast::Statement::new_label(name),
            _ => {
                self.lookahead_queue.push_back(token);
                let expr = self.expr(0)?;
                ast::Statement::new_expr(expr)
            }
        };
        // Consume optional EOS
        self.next_token_is(&Token::EndOfStatement)?;
        Ok(statement)
    }

    /// Get the next expression, possibly recurring to handle nested
    /// expressions, unary & binary expressions, blocks, functions, etc.
    fn expr(&mut self, prec: u8) -> ExprResult {
        let token = self.expect_next_token()?;
        let mut expr = match token.token {
            Token::LeftParen => self.nested_expr()?,
            Token::RightParen => {
                // XXX: The scanner detects mismatched brackets and
                //      self.nested_expr() handles right parens, so this
                //      will only happen when an empty group is
                //      encountered.
                return Err(ParseErr::new(ParseErrKind::ExpectedExpr(token.start)));
            }
            Token::Nil => ast::Expr::new_literal(ast::Literal::new_nil()),
            Token::True => ast::Expr::new_literal(ast::Literal::new_bool(true)),
            Token::False => ast::Expr::new_literal(ast::Literal::new_bool(false)),
            Token::Float(value) => {
                ast::Expr::new_literal(ast::Literal::new_float(value))
            }
            Token::Int(value) => ast::Expr::new_literal(ast::Literal::new_int(value)),
            Token::String(value) => {
                ast::Expr::new_literal(ast::Literal::new_string(value))
            }
            Token::FormatString(value) => {
                ast::Expr::new_literal(ast::Literal::new_format_string(value))
            }
            Token::Ident(name) => {
                if self.next_token_is(&Token::LeftParen)? {
                    // Function def or call
                    return Ok(self.func(name)?);
                }
                ast::Expr::new_ident(ast::Ident::new_ident(name))
            }
            Token::Block => {
                return Ok(self.block()?);
            }
            _ => self.expect_unary_expr(&token)?,
        };
        expr = self.maybe_binary_expr(prec, expr)?;
        // if self.next_token_is(&Token::Comma) {
        //     expr = self.tuple(expr)?;
        // }
        Ok(expr)
    }

    // fn tuple(&mut self, first_expr: ast::Expr) -> ExprResult {
    //     let mut items = vec![first_expr];
    //     loop {
    //         if self.is_eos()? {
    //             break;
    //         }
    //         let expr = self.expr(0)?;
    //         items.push(expr);
    //     }
    //     Ok(ast::Expr::new_tuple(items))
    // }

    /// The current token should represent a unary operator and should
    /// be followed by an expression.
    fn expect_unary_expr(&mut self, op_token: &TokenWithLocation) -> ExprResult {
        let prec = get_unary_precedence(&op_token.token);
        if prec == 0 {
            return Err(ParseErr::new(ParseErrKind::UnexpectedToken(op_token.clone())));
        }
        if !self.has_tokens()? {
            return Err(ParseErr::new(ParseErrKind::ExpectedOperand(op_token.end)));
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
                    return Err(ParseErr::new(ParseErrKind::ExpectedOperand(
                        infix_token.end,
                    )));
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
            return Err(ParseErr::new(ParseErrKind::ExpectedExpr(self.next_loc())));
        }
        let expr = self.expr(0)?;
        if self.next_token_is(&Token::RightParen)? {
            return Ok(expr);
        }
        self.nested_expr()
    }

    /// Handle `block ->`.
    fn block(&mut self) -> ExprResult {
        self.expect_token(&Token::FuncStart)?;
        self.expect_scope()?;
        let statements = self.expect_statement_block()?;
        Ok(ast::Expr::new_block(statements))
    }

    /// Handle `func () -> ...` (definition) and `func()` (call).
    fn func(&mut self, name: String) -> ExprResult {
        let (found, tokens) = self.collect_until(Token::RightParen)?;
        if !found {
            self.lookahead_queue.extend(tokens);
            return Err(ParseErr::new(ParseErrKind::ExpectedToken(
                self.next_loc(),
                Token::RightParen,
            )));
        }
        if self.next_token_is(&Token::FuncStart)? {
            // Function def - tokens are parameters
            let params = self.parse_params(tokens)?;
            self.expect_scope()?;
            let statements = self.expect_statement_block()?;
            Ok(ast::Expr::new_func(name.clone(), params, statements))
        } else {
            // Function call -- tokens are args
            let args = parse_tokens(tokens, false)?;
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
        let program = parse_tokens(tokens, false)?;
        for statement in program.statements {
            let kind = statement.kind;
            if let ast::StatementKind::Expr(ast::Expr {
                kind:
                    ast::ExprKind::Ident(ast::Ident { kind: ast::IdentKind::Ident(name) }),
            }) = kind
            {
                params.push(name);
            } else {
                return Err(ParseErr::new(ParseErrKind::SyntaxError(
                    "Expected identifier".to_owned(),
                    self.next_loc(),
                )));
            }
        }
        Ok(params)
    }
}
