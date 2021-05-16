use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};
use std::iter::Peekable;

use num_bigint::BigInt;

use crate::ast;
use crate::scanner::{self, ScanError, Scanner, Token, TokenWithLocation};

use super::{ParseError, ParseErrorKind, ParseResult};

/// Create a parser from the specified text, scan the text into tokens,
/// parse the tokens, and return the resulting AST or error.
pub fn parse_text(text: &str) -> ParseResult {
    let scanner = Scanner::<Cursor<&str>>::from_text(text);
    let mut parser = Parser::new(scanner);
    parser.parse()
}

/// Create a parser from the specified file, scan its text into tokens,
/// parse the tokens, and return the resulting AST or error.
pub fn parse_file(file_path: &str) -> ParseResult {
    let result = Scanner::<BufReader<File>>::from_file(file_path);
    let scanner = match result {
        Ok(scanner) => scanner,
        Err(err) => {
            return Err(ParseError::new(ParseErrorKind::CouldNotOpenSourceFile(
                err.to_string(),
            )));
        }
    };
    let mut parser = Parser::new(scanner);
    parser.parse()
}

pub struct Parser<T: BufRead> {
    token_stream: Peekable<Scanner<T>>,

    /// Keep track of tokens until a valid statement is encountered.
    /// TODO: ???
    token_queue: VecDeque<TokenWithLocation>,
}

impl<T: BufRead> Parser<T> {
    fn new(scanner: Scanner<T>) -> Self {
        Self { token_stream: scanner.peekable(), token_queue: VecDeque::new() }
    }

    /// Scan source -> tokens
    /// Parse tokens -> AST
    /// Walk AST -> instructions
    fn parse(&mut self) -> ParseResult {
        // A program is a list of statements.
        let statements = self.statements()?;
        let program = ast::Program::new(statements);
        Ok(program)
    }

    fn next_token(&mut self) -> Result<Option<TokenWithLocation>, ParseError> {
        match self.token_stream.next() {
            Some(result) => match result {
                Ok(token_with_location) => Ok(Some(token_with_location)),
                Err(err) => Err(self.scan_err(err)),
            },
            None => Ok(None),
        }
    }

    fn next_token_if(
        &mut self,
        func: impl FnOnce(&Token) -> bool,
    ) -> Result<Option<TokenWithLocation>, ParseError> {
        if let Some(t) = self.peek_token()? {
            if func(&t.token) {
                return Ok(self.next_token()?);
            }
        }
        Ok(None)
    }

    fn peek_token(&mut self) -> Result<Option<&TokenWithLocation>, ParseError> {
        match self.token_stream.peek() {
            Some(result) => match result {
                Ok(token_with_location) => Ok(Some(token_with_location)),
                Err(err) => {
                    Err(ParseError::new(ParseErrorKind::ScanError(err.clone())))
                }
            },
            None => Ok(None),
        }
    }

    /// Create a new ParseError of the specified kind.
    fn err(&self, kind: ParseErrorKind) -> ParseError {
        ParseError::new(kind)
    }

    /// Create a new ParseError that wraps a ScanError.
    fn scan_err(&self, err: ScanError) -> ParseError {
        self.err(ParseErrorKind::ScanError(err))
    }

    // Grammar

    #[rustfmt::skip]
    fn statements(&mut self) -> Result<Vec<ast::Statement>, ParseError> {
        let mut statements = vec![];
        loop {
            match self.expression()? {
                Some(expr) => {
                    let statement = ast::Statement::new_expression(expr);
                    statements.push(statement);
                },
                None => {
                    break Ok(statements);
                }
            }
        }
    }

    fn expression(&mut self) -> Result<Option<ast::Expression>, ParseError> {
        let token = match self.next_token()? {
            Some(token) => token,
            None => return Ok(None),
        };

        // *Always* start with a prefix expression, which includes
        // unary operations like -1 and !true as well as variable names.
        let lhs = match token.token {
            Token::Float(value) => {
                ast::Expression::new_literal(ast::Literal::new_float(value))
            }
            Token::Int(value) => {
                ast::Expression::new_literal(ast::Literal::new_int(value))
            }
            _ => return Err(self.err(ParseErrorKind::UnhandledToken(token))),
        };

        let maybe_infix_token = self.next_token_if(|token| match token {
            Token::Star | Token::Slash | Token::Plus | Token::Minus => true,
            _ => false,
        })?;

        // See if the expression from above is followed by an infix
        // operator. If so, get the RHS expression and return a binary
        // operation. If not, just return the original expression.
        let result = if let Some(infix_token) = maybe_infix_token {
            match self.expression()? {
                Some(rhs) => {
                    let op = infix_token.token.as_str();
                    ast::Expression::new_binary_operation(op, lhs, rhs)
                }
                None => return Err(self.err(ParseErrorKind::ExpectedExpression)),
            }
        } else {
            lhs
        };

        Ok(Some(result))
    }
}

#[cfg(test)]
mod tests {
    // All these tests except empty input hang, so they're ignored for
    // the time being.

    use super::*;

    #[test]
    fn parse_empty() {
        let result = parse_text("");
        assert!(result.is_ok());
        let program = result.unwrap();
        assert_eq!(program.statements.len(), 0);
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
                kind: ast::StatementKind::Expression(
                    Box::new(
                        ast::Expression {
                            kind: ast::ExpressionKind::Literal(
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
        parse_text("n = 1");
        assert!(false);
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
        
        // FIXME: This test passes, but only because it's testing what
        //        we know is being returned instead of what *should* be
        //        returned.
        assert_eq!(
            *statement,
            ast::Statement {
                kind: ast::StatementKind::Expression(
                    Box::new(
                        ast::Expression {
                            kind: ast::ExpressionKind::Literal(
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
    fn parse_assign_to_addition() {
        parse_text("n = 1 + 2");
        assert!(false);
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
        parse_text("a = 1\nb = a + 2\n");
        assert!(false);
    }
}
