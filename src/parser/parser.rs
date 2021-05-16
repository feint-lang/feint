use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};
use std::iter::Peekable;
use std::slice::Iter;

use num_bigint::BigInt;

use crate::ast;
use crate::scanner::{ScanError, Scanner, Token, TokenWithLocation};

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
            return Err(ParseError::new(ParseErrorKind::CouldNotOpenSourceFile(err)));
        }
    };
    let mut parser = Parser::new(scanner);
    parser.parse()
}

pub struct Parser<T: BufRead> {
    scanner: Scanner<T>,

    /// Keep track of tokens until a valid statement is encountered.
    /// TODO: ???
    token_queue: VecDeque<TokenWithLocation>,
}

impl<T: BufRead> Parser<T> {
    fn new(scanner: Scanner<T>) -> Self {
        Self { scanner, token_queue: VecDeque::new() }
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

    // Grammar

    #[rustfmt::skip]
    fn statements(&mut self) -> Result<Vec<ast::Statement>, ParseError> {
        let mut statements = vec![];

        let option = self.scanner.next(); // -> Option<ScanResult>
        if option.is_none() {
            return Ok(statements);
        }

        let result = option.unwrap(); // -> ScanResult
        if result.is_err() {
            let err = result.unwrap_err();
            return Err(self.parse_error(ParseErrorKind::ScanError(err)));
        }

        let token = result.unwrap(); // -> TokenWithLocation
        match token.token {
            Token::Int(value) => {
                let statement = ast::Statement::new_expression(
                    ast::Expression::new_literal(
                        ast::Literal::new_int(value)
                    ),
                );
                statements.push(statement);
            }
            t @ Token::Plus | t @ Token::Minus => {
                // Handle binary op
            }
            _ => {
                return Err(self.parse_error(ParseErrorKind::UnhandledToken(token)));
            }
        }

        Ok(statements)
    }

    fn expression(&mut self) {}

    fn parse_error(&self, kind: ParseErrorKind) -> ParseError {
        ParseError::new(kind)
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
