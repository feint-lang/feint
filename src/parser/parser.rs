use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Cursor};
use std::iter::Peekable;

use crate::ast;
use crate::scanner::{ScanError, Scanner, Token, TokenWithLocation};
use crate::util::Location;

use super::precedence::{
    get_binary_precedence, get_unary_precedence, is_right_associative,
};
use super::result::{
    ExprOptionResult, NextTokenResult, ParseError, ParseErrorKind, ParseResult,
    StatementResult,
};
use crate::parser::result::{
    ExprResult, NextInfixResult, PeekTokenResult, StatementsResult,
};

/// Create a parser from the specified text, scan the text into tokens,
/// parse the tokens, and return the resulting AST or error.
pub fn parse_text(text: &str) -> ParseResult {
    let mut parser = Parser::<Cursor<&str>>::from_text(text);
    parser.parse()
}

/// Create a parser from the specified file, scan its text into tokens,
/// parse the tokens, and return the resulting AST or error.
pub fn parse_file(file_path: &str) -> ParseResult {
    let mut parser = Parser::<BufReader<File>>::from_file(file_path)?;
    parser.parse()
}

/// Create a parser from stdin, scan the text into tokens, parse the
/// tokens, and return the resulting AST or error.
pub fn parse_stdin() -> ParseResult {
    let mut parser = Parser::<BufReader<io::Stdin>>::from_stdin();
    parser.parse()
}

struct Parser<T: BufRead> {
    token_stream: Peekable<Scanner<T>>,

    /// Keep track of tokens until a valid statement is encountered.
    /// TODO: ???
    token_queue: VecDeque<TokenWithLocation>,

    expecting_block: bool,
    precedence: u8,
}

impl<T: BufRead> Parser<T> {
    fn new(scanner: Scanner<T>) -> Self {
        Self {
            token_stream: scanner.peekable(),
            token_queue: VecDeque::new(),
            expecting_block: false,
            precedence: 0,
        }
    }

    pub fn from_text(text: &str) -> Parser<Cursor<&str>> {
        let scanner = Scanner::<Cursor<&str>>::from_text(text);
        Parser::new(scanner)
    }

    pub fn from_file(file_path: &str) -> Result<Parser<BufReader<File>>, ParseError> {
        let result = Scanner::<BufReader<File>>::from_file(file_path);
        let scanner = match result {
            Ok(scanner) => scanner,
            Err(err) => {
                return Err(ParseError::new(ParseErrorKind::CouldNotOpenSourceFile(
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

    fn next_token(&mut self) -> NextTokenResult {
        if let Some(result) = self.token_stream.next() {
            return result
                .map(|token_with_location| Some(token_with_location))
                .map_err(|err| ParseError::new(ParseErrorKind::ScanError(err.clone())));
        }
        Ok(None)
    }

    fn next_token_if(&mut self, func: impl FnOnce(&Token) -> bool) -> NextTokenResult {
        if let Some(t) = self.peek_token()? {
            if func(&t.token) {
                return Ok(self.next_token()?);
            }
        }
        Ok(None)
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
        // peek() returns Option<ScanResult>
        if let Some(result) = self.token_stream.peek() {
            return result
                .as_ref()
                .map(|token_with_location| Some(token_with_location))
                .map_err(|err| ParseError::new(ParseErrorKind::ScanError(err.clone())));
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

    // Error utilities -------------------------------------------------

    /// Create a new ParseError of the specified kind.
    fn err(&self, kind: ParseErrorKind) -> ParseError {
        ParseError::new(kind)
    }

    // Grammar ---------------------------------------------------------

    fn statements(&mut self) -> StatementsResult {
        let mut statements = vec![];
        loop {
            self.precedence = 0;
            let peek_result = self.peek_token()?;
            if let Some(token) = peek_result {
                match &token.token {
                    Token::BlockEnd => {
                        self.next_token()?;
                        self.expecting_block = false;
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
                    Token::Label(name) => {
                        statements.push(ast::Statement::new_label(name));
                        self.next_token()?;
                    }
                    Token::Jump => {
                        self.next_token()?;
                        if let Some(token) = self.next_token()? {
                            match token.token {
                                Token::Ident(name) => {
                                    let st = ast::Statement::new_jump_to_label(name);
                                    statements.push(st);
                                }
                                _ => {
                                    return Err(self.err(
                                        ParseErrorKind::ExpectedIdentifier(
                                            "jump label".to_owned(),
                                        ),
                                    ))
                                }
                            }
                        };
                    }
                    _ => {
                        if let Some(expr) = self.expr()? {
                            let statement = ast::Statement::new_expr(expr);
                            statements.push(statement);
                        }
                    }
                }
            } else {
                break;
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
            Token::Ident(name) => ast::Expr::new_ident(ast::Ident::new_ident(name)),
            Token::Block => {
                self.expecting_block = true;
                return self.block(token.end);
            }
            Token::BlockStart => {
                if !self.expecting_block {
                    return Err(self.err(ParseErrorKind::UnexpectedBlock(token.end)));
                }
                let statements = self.statements()?;
                ast::Expr::new_block(statements)
            }
            // The token isn't a leaf node, so it *must* be some other
            // kind of prefix token--a unary operation like -1 or !true.
            _ => {
                let precedence = get_unary_precedence(&token.token);
                if precedence == 0 {
                    return Err(self.err(ParseErrorKind::UnhandledToken(token.clone())));
                }
                if let Some(rhs) = self.expr()? {
                    let operator = token.token.as_str();
                    return Ok(Some(ast::Expr::new_unary_op(operator, rhs)));
                } else {
                    return Err(self.err(ParseErrorKind::ExpectedExpression(token.end)));
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
                    return Err(
                        self.err(ParseErrorKind::ExpectedExpression(infix_token.end))
                    );
                }
            } else {
                break;
            }
        }

        Ok(Some(expr))
    }

    fn block(&mut self, end: Location) -> ExprOptionResult {
        if let Ok(Some(_)) = self.next_token_if(|t| t == &Token::FuncStart) {
            if let Ok(Some(_)) = self.next_token_if(|t| t == &Token::EndOfStatement) {
                if let Ok(Some(_)) = self.peek_token_if(|t| t == &Token::BlockStart) {
                    self.expr()
                } else {
                    Err(self.err(ParseErrorKind::SyntaxError(
                        "Expected block".to_owned(),
                        Location::new(end.line + 1, 1),
                    )))
                }
            } else {
                Err(self.err(ParseErrorKind::SyntaxError(
                    "Expected end of line after ->".to_owned(),
                    Location::new(end.line, end.col + 4),
                )))
            }
        } else {
            Err(self.err(ParseErrorKind::SyntaxError(
                "Expected ->".to_owned(),
                Location::new(end.line, end.col + 2),
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::BinaryOperator;
    use num_bigint::BigInt;

    #[test]
    fn parse_empty() {
        let result = parse_text("");
        if let Ok(program) = result {
            assert_eq!(program.statements.len(), 0);
        } else {
            assert!(false, "Program failed to parse: {:?}", result);
        }
    }

    #[test]
    #[rustfmt::skip]
    fn parse_int() {
        let result = parse_text("1");
        assert!(result.is_ok());
        let program = result.unwrap();
        let statements = program.statements;
        assert_eq!(statements.len(), 1);
        let statement = statements.first().unwrap();
        assert_eq!(
            *statement,
            ast::Statement {
                kind: ast::StatementKind::Expr(
                    Box::new(
                        ast::Expr {
                            kind: ast::ExprKind::Literal(
                                Box::new(
                                    ast::Literal {
                                        kind: ast::LiteralKind::Int(
                                            BigInt::from(1)
                                        )
                                    }
                                )
                            )
                        }
                    )
                )
            }
        );
    }

    #[test]
    fn parse_simple_assignment() {
        //      R
        //      |
        //      n=
        //      |
        //      1
        let result = parse_text("n = 1");
        if let Ok(program) = result {
            assert_eq!(program.statements.len(), 1);
            // TODO: More checks
        } else {
            assert!(false, "Program failed to parse: {:?}", result);
        }
    }

    #[test]
    #[rustfmt::skip]
    fn parse_add() {
        //      R
        //      |
        //      +
        //     / \
        //    1   2
        let result = parse_text("1 + 2");
        assert!(result.is_ok());
        let program = result.unwrap();
        let statements = program.statements;
        assert_eq!(statements.len(), 1);
        let statement = statements.first().unwrap();

        assert_eq!(
            *statement,
            ast::Statement {
                kind: ast::StatementKind::Expr(
                    Box::new(
                        // 1 + 2
                        ast::Expr {
                            kind: ast::ExprKind::BinaryOp(
                                Box::new(
                                    // 1
                                    ast::Expr {
                                        kind: ast::ExprKind::Literal(
                                            Box::new(
                                                ast::Literal {
                                                    kind: ast::LiteralKind::Int(BigInt::from(1))
                                                }
                                            )
                                        )
                                    }
                                ),
                                // +
                                BinaryOperator::Add,
                                Box::new(
                                    // 2
                                    ast::Expr {
                                        kind: ast::ExprKind::Literal(
                                            Box::new(
                                                ast::Literal {
                                                    kind: ast::LiteralKind::Int(BigInt::from(2))
                                                }
                                            )
                                        )
                                    }
                                ),
                            )
                        }
                    )
                )
            }
        );
    }

    #[test]
    fn parse_assign_to_addition() {
        let result = parse_text("n = 1 + 2");
        if let Ok(program) = result {
            assert_eq!(program.statements.len(), 1);
            eprintln!("{:?}", program);
            // TODO: More checks
        } else {
            assert!(false, "Program failed to parse: {:?}", result);
        }
    }

    #[test]
    fn parse_simple_program() {
        //      ROOT
        //     /    \
        //    a=    b=
        //    |     |
        //    1     +
        //         / \
        //        a   1
        let result = parse_text("a = 1\nb = a + 2\n");
        if let Ok(program) = result {
            assert_eq!(program.statements.len(), 2);
            // TODO: More checks
        } else {
            assert!(false, "Program failed to parse");
        }
    }
}
