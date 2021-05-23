use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Cursor};
use std::iter::Peekable;

use crate::ast;
use crate::scanner::{ScanError, Scanner, Token, TokenWithLocation};

use super::precedence::{
    get_binary_precedence, get_unary_precedence, is_right_associative,
};
use super::{ParseError, ParseErrorKind, ParseResult};

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
}

impl<T: BufRead> Parser<T> {
    fn new(scanner: Scanner<T>) -> Self {
        Self { token_stream: scanner.peekable(), token_queue: VecDeque::new() }
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
            if self.peek_token()?.is_none() {
                break;
            }
            if let Some(expr) = self.expr(0)? {
                let statement = ast::Statement::new_expr(expr);
                statements.push(statement);
            }
        }
        Ok(statements)
    }

    fn expr(&mut self, precedence: u8) -> Result<Option<ast::Expr>, ParseError> {
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
            Token::Float(value) => {
                ast::Expr::new_literal(ast::Literal::new_float(value))
            }
            Token::Int(value) => ast::Expr::new_literal(ast::Literal::new_int(value)),
            Token::Identifier(name) => {
                ast::Expr::new_identifier(ast::Identifier::new_identifier(name))
            }
            // The token isn't a leaf node, so it *must* be some other
            // kind of prefix token--a unary operation like -1 or !true.
            _ => self.unary_expression(&token)?,
        };

        // See if the expr from above is followed by an infix
        // operator. If so, get the RHS expr and return a binary
        // operation. If not, just return the original expr.
        loop {
            let next = self.next_infix_token(precedence)?;
            if let Some((infix_token, mut infix_precedence)) = next {
                // Lower precedence of right-associative operator when
                // fetching its RHS expr.
                if is_right_associative(&infix_token.token) {
                    infix_precedence -= 1;
                }
                if let Some(rhs) = self.expr(infix_precedence)? {
                    let op = infix_token.token.as_str();
                    expr = ast::Expr::new_binary_operation(expr, op, rhs);
                } else {
                    return Err(
                        self.err(ParseErrorKind::ExpectedExpression(infix_token))
                    );
                }
            } else {
                break;
            }
        }

        Ok(Some(expr))
    }

    /// Get unary expr for the current unary operator token.
    fn unary_expression(
        &mut self,
        token: &TokenWithLocation,
    ) -> Result<ast::Expr, ParseError> {
        let precedence = get_unary_precedence(&token.token);
        if precedence == 0 {
            return Err(self.err(ParseErrorKind::UnhandledToken(token.clone())));
        }
        if let Some(rhs) = self.expr(precedence)? {
            let operator = token.token.as_str();
            return Ok(ast::Expr::new_unary_operation(operator, rhs));
        }
        return Err(self.err(ParseErrorKind::ExpectedExpression(token.clone())));
    }

    /// Return the next token along with its precedence *if* it's both
    /// an infix operator *and* its precedence is greater than the
    /// current precedence level.
    fn next_infix_token(
        &mut self,
        current_precedence: u8,
    ) -> Result<Option<(TokenWithLocation, u8)>, ParseError> {
        if let Some(token) = self.next_token_if(|t| {
            let p = get_binary_precedence(t);
            p > 0 && p > current_precedence
        })? {
            let precedence = get_binary_precedence(&token.token);
            return Ok(Some((token, precedence)));
        }
        Ok(None)
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
                            kind: ast::ExprKind::BinaryOperation(
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
