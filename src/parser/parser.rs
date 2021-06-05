use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Cursor};
use std::iter::Peekable;

use crate::ast;
use crate::scanner::{Scanner, Token, TokenWithLocation};
use crate::util::Location;

use super::precedence::{
    get_binary_precedence, get_unary_precedence, is_right_associative,
};
use super::result::{
    ExprOptionResult, ExprResult, NextTokenResult, ParseErr, ParseErrKind, ParseResult,
};
use crate::parser::result::{NextInfixResult, PeekTokenResult, StatementsResult};

/// Create a parser from the specified text, scan the text into tokens,
/// parse the tokens, and return the resulting AST or error.
pub fn parse_text(text: &str, debug: bool) -> ParseResult {
    let mut parser = Parser::<Cursor<&str>>::from_text(text);
    handle_result(parser.parse(), debug)
}

/// Create a parser from the specified file, scan its text into tokens,
/// parse the tokens, and return the resulting AST or error.
pub fn parse_file(file_path: &str, debug: bool) -> ParseResult {
    let mut parser = Parser::<BufReader<File>>::from_file(file_path)?;
    handle_result(parser.parse(), debug)
}

/// Create a parser from stdin, scan the text into tokens, parse the
/// tokens, and return the resulting AST or error.
pub fn parse_stdin(debug: bool) -> ParseResult {
    let mut parser = Parser::<BufReader<io::Stdin>>::from_stdin();
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

struct Parser<T: BufRead> {
    current_token: Option<TokenWithLocation>,
    token_stream: Peekable<Scanner<T>>,
    lookahead_queue: VecDeque<TokenWithLocation>,
}

impl<T: BufRead> Parser<T> {
    fn new(scanner: Scanner<T>) -> Self {
        Self {
            current_token: None,
            token_stream: scanner.peekable(),
            lookahead_queue: VecDeque::new(),
        }
    }

    pub fn from_text(text: &str) -> Parser<Cursor<&str>> {
        let scanner = Scanner::<Cursor<&str>>::from_text(text);
        Parser::new(scanner)
    }

    pub fn from_file(file_path: &str) -> Result<Parser<BufReader<File>>, ParseErr> {
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
        Ok(Parser::new(scanner))
    }

    pub fn from_stdin() -> Parser<BufReader<io::Stdin>> {
        let scanner = Scanner::<BufReader<io::Stdin>>::from_stdin();
        Parser::new(scanner)
    }

    // Parse entry point -----------------------------------------------

    /// Scan source -> tokens
    /// Parse tokens -> AST
    /// Walk AST -> instructions
    fn parse(&mut self) -> ParseResult {
        // A program is a list of statements.
        let statements = self.statements()?;
        let program = ast::Program::new(statements);
        Ok(program)
    }

    // Tokens ----------------------------------------------------------

    fn loc(&self) -> Location {
        match &self.current_token {
            Some(token) => token.start,
            None => Location::new(0, 0),
        }
    }

    fn next_token(&mut self) -> NextTokenResult {
        if let Some(token_with_location) = self.lookahead_queue.pop_front() {
            self.current_token = Some(token_with_location.clone());
            return Ok(Some(token_with_location));
        } else if let Some(result) = self.token_stream.next() {
            return match result {
                Ok(token_with_location) => {
                    self.current_token = Some(token_with_location.clone());
                    Ok(Some(token_with_location))
                }
                Err(err) => Err(ParseErr::new(ParseErrKind::ScanError(err))),
            };
        }
        Ok(None)
    }

    /// Consume the next token and return it if the specified condition
    /// is true. Otherwise, return None.
    fn next_token_if(&mut self, func: impl FnOnce(&Token) -> bool) -> NextTokenResult {
        if let Some(t) = self.peek_token()? {
            if func(&t.token) {
                return Ok(self.next_token()?);
            }
        }
        Ok(None)
    }

    /// Consume next token and return true if next token is equal to
    /// specified token. Otherwise, leave the token in the stream and
    /// return false.
    fn next_token_is(&mut self, token: Token) -> Result<bool, ParseErr> {
        if let Some(_) = self.next_token_if(|next| next == &token)? {
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

    fn peek_token(&mut self) -> PeekTokenResult {
        if let Some(token_with_location) = self.lookahead_queue.front() {
            return Ok(Some(token_with_location));
        } else if let Some(result) = self.token_stream.peek() {
            // token_stream.peek() returns Option<ScanResult>
            return result
                .as_ref()
                .map(|token_with_location| Some(token_with_location))
                .map_err(|err| ParseErr::new(ParseErrKind::ScanError(err.clone())));
        }
        Ok(None)
    }

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
    fn peek_token_is(&mut self, token: Token) -> Result<bool, ParseErr> {
        if let Some(_) = self.peek_token_if(|next| next == &token)? {
            return Ok(true);
        }
        Ok(false)
    }

    // Utilities -------------------------------------------------------

    fn collect_until(
        &mut self,
        token: Token,
    ) -> Result<(bool, Vec<TokenWithLocation>), ParseErr> {
        let mut collector = vec![];
        while let Some(t) = self.next_token()? {
            if t.token == token {
                return Ok((true, collector));
            }
            collector.push(t);
        }
        Ok((false, collector))
    }

    fn expect_block(&mut self) -> Result<(), ParseErr> {
        let end_of_statement = self.next_token_is(Token::EndOfStatement)?;
        if !(end_of_statement && self.next_token_is(Token::ScopeStart)?) {
            return Err(ParseErr::new(ParseErrKind::ExpectedBlock(self.loc())));
        }
        Ok(())
    }

    fn expect_statements(&mut self) -> StatementsResult {
        let statements = self.statements()?;
        if statements.is_empty() {
            return Err(ParseErr::new(ParseErrKind::ExpectedBlock(self.loc())));
        }
        Ok(statements)
    }

    // Grammar ---------------------------------------------------------

    fn statements(&mut self) -> StatementsResult {
        let mut statements = vec![];
        loop {
            let token = match self.peek_token()? {
                Some(token) => token.token.clone(),
                None => break,
            };
            match token {
                Token::ScopeEnd => {
                    self.next_token()?;
                    break;
                }
                Token::Print => {
                    self.next_token()?;
                    let statement = match self.expr(0)? {
                        Some(expr) => ast::Statement::new_expr(expr),
                        None => ast::Statement::new_string(""),
                    };
                    statements.push(statement);
                    statements.push(ast::Statement::new_print());
                }
                Token::Jump => {
                    self.next_token()?;
                    if let Some(token) = self.next_token()? {
                        match token.token {
                            Token::Ident(name) => {
                                statements.push(ast::Statement::new_jump(name));
                            }
                            _ => {
                                return Err(ParseErr::new(
                                    ParseErrKind::ExpectedIdent(token),
                                ));
                            }
                        }
                    };
                }
                Token::Label(name) => {
                    self.next_token()?;
                    statements.push(ast::Statement::new_label(name));
                }
                _ => {
                    if let Some(expr) = self.expr(0)? {
                        let statement = ast::Statement::new_expr(expr);
                        statements.push(statement);
                    }
                }
            }
        }
        Ok(statements)
    }

    fn expr(&mut self, prec: u8) -> ExprOptionResult {
        let token = match self.next_token()? {
            Some(token) => token,
            None => return Ok(None),
        };

        let expr = match token.token {
            Token::EndOfStatement => {
                return Ok(None);
            }
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
                if self.next_token_is(Token::LeftParen)? {
                    // Function def or call
                    return Ok(Some(self.func(name)?));
                }
                ast::Expr::new_ident(ast::Ident::new_ident(name))
            }
            Token::Block => {
                return Ok(Some(self.block()?));
            }
            Token::FuncStart | Token::ScopeStart => {
                // XXX: This should only happened when an otherwise
                //      unhandled start token is encountered.
                return Err(ParseErr::new(ParseErrKind::UnexpectedBlock(token.end)));
            }
            _ => self.expect_unary_expr(&token)?,
        };

        let expr = self.maybe_binary_expr(prec, expr)?;

        Ok(Some(expr))
    }

    // The current token should represent a unary operator and should be
    // followed by an expression.
    fn expect_unary_expr(&mut self, token: &TokenWithLocation) -> ExprResult {
        let prec = get_unary_precedence(&token.token);
        if prec == 0 {
            Err(ParseErr::new(ParseErrKind::UnhandledToken(token.clone())))
        } else if let Some(rhs) = self.expr(prec)? {
            let operator = token.token.as_str();
            Ok(ast::Expr::new_unary_op(operator, rhs))
        } else {
            Err(ParseErr::new(ParseErrKind::ExpectedOperand(token.end)))
        }
    }

    // See if the expr is followed by an infix operator. If so, get the
    // RHS expr and return a binary expression. If not, just return the
    // original expr.
    fn maybe_binary_expr(&mut self, prec: u8, mut expr: ast::Expr) -> ExprResult {
        loop {
            let next = self.next_infix_token(prec)?;
            if let Some((infix_token, mut infix_prec)) = next {
                // Lower precedence of right-associative operator when
                // fetching its RHS expr.
                if is_right_associative(&infix_token.token) {
                    infix_prec -= 1;
                }
                if let Some(rhs) = self.expr(infix_prec)? {
                    let op = infix_token.token.as_str();
                    expr = ast::Expr::new_binary_op(expr, op, rhs);
                } else {
                    return Err(ParseErr::new(ParseErrKind::ExpectedOperand(
                        infix_token.end,
                    )));
                }
            } else {
                break Ok(expr);
            }
        }
    }

    fn nested_expr(&mut self) -> ExprResult {
        return match self.expr(0)? {
            Some(mut expr) => {
                if self.next_token_is(Token::RightParen)? {
                    return Ok(expr);
                }
                self.nested_expr()
            }
            None => Err(ParseErr::new(ParseErrKind::ExpectedExpr(self.loc()))),
        };
    }

    fn block(&mut self) -> ExprResult {
        if !self.next_token_is(Token::FuncStart)? {
            return Err(ParseErr::new(ParseErrKind::SyntaxError(
                "Expected ->".to_owned(),
                self.loc(),
            )));
        }
        self.expect_block()?;
        Ok(ast::Expr::new_block(self.expect_statements()?))
    }

    fn func(&mut self, name: String) -> ExprResult {
        let loc = self.loc();
        let (found, tokens) = self.collect_until(Token::RightParen)?;
        if !found {
            self.lookahead_queue.extend(tokens);
            return Err(ParseErr::new(ParseErrKind::ExpectedToken(
                loc,
                Token::RightParen,
            )));
        }
        if self.next_token_is(Token::FuncStart)? {
            // Function def -- tokens are parameters
            self.expect_block()?;
            let statements = self.expect_statements()?;
            Ok(ast::Expr::new_func(name.clone(), statements))
        } else {
            // Function call -- tokens are args
            // FIXME: Temporary
            Ok(ast::Expr::new_func(name.clone(), vec![]))
        }
    }
}
