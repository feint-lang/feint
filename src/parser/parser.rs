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
    /// Current operator precedence
    precedence: u8,
    expecting_block: bool,
}

impl<T: BufRead> Parser<T> {
    fn new(scanner: Scanner<T>) -> Self {
        Self {
            current_token: None,
            token_stream: scanner.peekable(),
            lookahead_queue: VecDeque::new(),
            precedence: 0,
            expecting_block: false,
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

    fn enter_scope(&mut self) {
        self.expecting_block = true;
    }

    fn exit_scope(&mut self) {
        self.expecting_block = false;
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
    fn next_infix_token(&mut self) -> NextInfixResult {
        let current_precedence = self.precedence;
        if let Some(token) = self.next_token_if(|t| {
            let p = get_binary_precedence(t);
            p > 0 && p > current_precedence
        })? {
            let precedence = get_binary_precedence(&token.token);
            return Ok(Some((token, precedence)));
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

    /// Create a new ParseError of the specified kind.
    fn err(&self, kind: ParseErrKind) -> ParseErr {
        ParseErr::new(kind)
    }

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
            return Err(self.err(ParseErrKind::ExpectedBlock(self.loc())));
        }
        Ok(())
    }

    fn expect_statements(&mut self) -> StatementsResult {
        let statements = self.statements()?;
        if statements.is_empty() {
            return Err(self.err(ParseErrKind::ExpectedBlock(self.loc())));
        }
        Ok(statements)
    }

    // Grammar ---------------------------------------------------------

    fn statements(&mut self) -> StatementsResult {
        let mut statements = vec![];
        loop {
            self.precedence = 0;
            let token = if let Some(token) = self.peek_token()? {
                token.token.clone()
            } else {
                break;
            };
            match token {
                Token::ScopeEnd => {
                    self.next_token()?;
                    self.exit_scope();
                    break;
                }
                Token::Print => {
                    self.next_token()?;
                    let statement = match self.expr()? {
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
                                return Err(self.err(ParseErrKind::ExpectedIdent(token)))
                            }
                        }
                    };
                }
                Token::Label(name) => {
                    self.next_token()?;
                    statements.push(ast::Statement::new_label(name));
                }
                _ => {
                    if let Some(expr) = self.expr()? {
                        let statement = ast::Statement::new_expr(expr);
                        statements.push(statement);
                    }
                }
            }
        }
        Ok(statements)
    }

    fn expr(&mut self) -> ExprOptionResult {
        let token = match self.next_token()? {
            Some(token) => token,
            None => return Ok(None),
        };

        let mut expr = match token.token {
            Token::EndOfStatement => {
                return Ok(None);
            }
            Token::LeftParen => {
                //
                let expr = self.expr()?;
                if !self.next_token_is(Token::RightParen)? {
                    return Err(self.err(ParseErrKind::UnclosedExpr(token.start)));
                }
                expr.unwrap()
            }
            // First, try for a literal or identifier, since they're
            // leaf nodes.
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
                // block ->
                //     ...
                return Ok(Some(self.block()?));
            }
            Token::FuncStart => {
                // XXX: This should only happened when an otherwise
                //      unhandled func start token is encountered.
                return Err(self.err(ParseErrKind::UnexpectedToken(token)));
            }
            Token::ScopeStart => {
                // XXX: This should only happened when an otherwise
                //      unhandled scope start token is encountered.
                return Err(self.err(ParseErrKind::UnexpectedBlock(token.end)));
            }
            // The token isn't a leaf node, so it *must* be some other
            // kind of prefix token--a unary operation like -1 or !true.
            _ => {
                let precedence = get_unary_precedence(&token.token);
                if precedence == 0 {
                    return Err(self.err(ParseErrKind::UnhandledToken(token.clone())));
                }
                if let Some(rhs) = self.expr()? {
                    let operator = token.token.as_str();
                    return Ok(Some(ast::Expr::new_unary_op(operator, rhs)));
                } else {
                    return Err(self.err(ParseErrKind::ExpectedExpr(token.end)));
                }
            }
        };

        // See if the expr from above is followed by an infix
        // operator. If so, get the RHS expr and return a binary
        // operation. If not, just return the original expr.
        loop {
            let next = self.next_infix_token()?;
            if let Some((infix_token, mut infix_precedence)) = next {
                // Lower precedence of right-associative operator when
                // fetching its RHS expr.
                if is_right_associative(&infix_token.token) {
                    infix_precedence -= 1;
                }
                self.precedence = infix_precedence;
                if let Some(rhs) = self.expr()? {
                    let op = infix_token.token.as_str();
                    expr = ast::Expr::new_binary_op(expr, op, rhs);
                } else {
                    return Err(self.err(ParseErrKind::ExpectedExpr(infix_token.end)));
                }
            } else {
                break;
            }
        }

        Ok(Some(expr))
    }

    fn block(&mut self) -> ExprResult {
        if !self.next_token_is(Token::FuncStart)? {
            return Err(self
                .err(ParseErrKind::SyntaxError("Expected ->".to_owned(), self.loc())));
        }
        self.expect_block()?;
        self.enter_scope();
        Ok(ast::Expr::new_block(self.expect_statements()?))
    }

    fn func(&mut self, name: String) -> ExprResult {
        let loc = self.loc();
        let (found, tokens) = self.collect_until(Token::RightParen)?;
        if !found {
            self.lookahead_queue.extend(tokens);
            return Err(self.err(ParseErrKind::ExpectedToken(loc, Token::RightParen)));
        }
        if self.next_token_is(Token::FuncStart)? {
            // Function def -- tokens are parameters
            self.expect_block()?;
            self.enter_scope();
            let statements = self.expect_statements()?;
            Ok(ast::Expr::new_func(name.clone(), statements))
        } else {
            // Function call -- tokens are args
            // FIXME: Temporary
            Ok(ast::Expr::new_func(name.clone(), vec![]))
        }
    }
}
