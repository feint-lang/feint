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
        if let Some(result) = self.token_stream.next() {
            return result
                .map(|token_with_location| Some(token_with_location))
                .map_err(|err| self.scan_err(err));
        }
        Ok(None)
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
        // peek() returns Option<ScanResult>
        if let Some(result) = self.token_stream.peek() {
            return result
                .as_ref()
                .map(|token_with_location| Some(token_with_location))
                .map_err(|err| {
                    // XXX: Can't use self.scan_err() here???
                    ParseError::new(ParseErrorKind::ScanError(err.clone()))
                });
        }
        Ok(None)
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
            match self.expression(0)? {
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

    fn expression(
        &mut self,
        precedence: u8,
    ) -> Result<Option<ast::Expression>, ParseError> {
        let token_with_location = match self.next_token()? {
            Some(t) => t,
            None => return Ok(None),
        };

        // *Always* start with a prefix expression, which includes
        // unary operations like -1 and !true as well as variable names.
        let mut lhs = match token_with_location.token {
            Token::Float(value) => {
                ast::Expression::new_literal(ast::Literal::new_float(value))
            }
            Token::Int(value) => {
                ast::Expression::new_literal(ast::Literal::new_int(value))
            }
            _ => {
                let token = &token_with_location.token;
                let unary_precedence = self.get_unary_precedence(token);
                if unary_precedence == 0 {
                    return Err(
                        self.err(ParseErrorKind::UnhandledToken(token_with_location))
                    );
                }
                match self.expression(unary_precedence)? {
                    Some(rhs) => {
                        let op = token.as_str();
                        ast::Expression::new_unary_operation(op, rhs)
                    }
                    None => return Err(self.err(ParseErrorKind::ExpectedExpression)),
                }
            }
        };

        // See if the expression from above is followed by an infix
        // operator. If so, get the RHS expression and return a binary
        // operation. If not, just return the original expression. Note
        // that when the next precedence is 0, that indicates that the
        // next token is *not* a binary operator.
        let mut next_precedence = self.get_next_binary_precedence()?;

        while precedence < next_precedence {
            let infix_token = self.next_token()?.unwrap();
            match self.expression(next_precedence)? {
                Some(rhs) => {
                    let op = infix_token.token.as_str();
                    lhs = ast::Expression::new_binary_operation(lhs, op, rhs);
                    next_precedence = self.get_next_binary_precedence()?;
                }
                None => return Err(self.err(ParseErrorKind::ExpectedExpression)),
            }
        }

        Ok(Some(lhs))
    }

    /// Get unary precedence of token.
    fn get_unary_precedence(&self, token: &Token) -> u8 {
        self.get_operator_precedence(token).0
    }

    /// Get binary precedence of token.
    fn get_binary_precedence(&self, token: &Token) -> u8 {
        self.get_operator_precedence(token).1
    }

    /// Get binary precedence of next token.
    fn get_next_binary_precedence(&mut self) -> Result<u8, ParseError> {
        if let Some(TokenWithLocation { token, .. }) = self.peek_token()? {
            /// FIXME: Usage of clone that feels wrong
            let token = token.clone();
            Ok(self.get_operator_precedence(&token).1)
        } else {
            Ok(0)
        }
    }

    #[rustfmt::skip]
    /// Return the unary *and* binary precedence of the specified token,
    /// which may be 0 for either or both. 0 indicates that the token is
    /// not an operator of the respective type.
    ///
    /// TODO: I'm not sure this is the best way to define this mapping.
    ///       Would a static hash map be better? One issue with that is
    ///       that Token can't be used as a hash map key, since it's not
    ///       hashable. That could probably be "fixed", but it would be
    ///       more complicated than this.
    fn get_operator_precedence(&self, token: &Token) -> (u8, u8) {
        match token {
            Token::Plus =>        (4, 1), // +a, a + b (no-op, addition)
            Token::Minus =>       (4, 1), // -a, a - b (negation, subtraction)
            Token::Star =>        (0, 2), // a * b     (multiplication)
            Token::Slash =>       (0, 2), // a / b     (division)
            Token::DoubleSlash => (0, 2), // a // b    (floor division)
            Token::Percent =>     (0, 2), // a % b     (modulus)
            Token::Caret =>       (0, 3), // a ^ b     (exponentiation)
            Token::Bang =>        (4, 0), // !a        (logical not)
            _ =>                  (0, 0), //           (not an operator)
        }
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
