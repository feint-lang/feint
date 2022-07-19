use std::io::Cursor;

use num_bigint::BigInt;

use crate::ast;
use crate::parser::*;
use crate::scanner::Scanner;
use crate::util::{source_from_text, BinaryOperator};

/// Scan the text into tokens, parse the tokens, and return the
/// resulting AST or error.
pub fn parse_text(text: &str) -> ParseResult {
    let mut source = source_from_text(text);
    let scanner = Scanner::new(&mut source);
    let mut parser = Parser::new(scanner.into_iter());
    parser.parse()
}

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
                    ast::Expr {
                        kind: ast::ExprKind::Literal(
                            ast::Literal {
                                kind: ast::LiteralKind::Int(
                                    BigInt::from(1)
                                )
                            }
                        )
                    }
                )
            }
        );
    }

#[test]
fn parse_inline_block() {
    let result = parse_text("block -> true");
    assert!(result.is_ok());
    let program = result.unwrap();
    let statements = program.statements;
    // eprintln!("{statements:?}");
    // check_token(tokens.next(), Token::Block, 1, 1, 1, 5);
    // check_token(tokens.next(), Token::ScopeStart, 1, 7, 1, 8);
    // check_token(tokens.next(), Token::True, 1, 10, 1, 13);
    // check_token(tokens.next(), Token::ScopeEnd, 1, 14, 1, 14);
    // check_token(tokens.next(), Token::EndOfStatement, 1, 14, 1, 14);
    // assert!(tokens.next().is_none());
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
fn parse_add() {
    //      R
    //      |
    //      +
    //     / \
    //    1   2
    let result = parse_text("1 + 2");
    assert!(result.is_ok(), "{:?}", result);
    let program = result.unwrap();
    let statements = program.statements;
    assert_eq!(statements.len(), 1);
    let statement = statements.first().unwrap();

    assert_eq!(
        *statement,
        ast::Statement {
            kind: ast::StatementKind::Expr(
                // 1 + 2
                ast::Expr {
                    kind: ast::ExprKind::BinaryOp(
                        Box::new(
                            // 1
                            ast::Expr {
                                kind: ast::ExprKind::Literal(ast::Literal {
                                    kind: ast::LiteralKind::Int(BigInt::from(1))
                                })
                            }
                        ),
                        // +
                        BinaryOperator::Add,
                        Box::new(
                            // 2
                            ast::Expr {
                                kind: ast::ExprKind::Literal(ast::Literal {
                                    kind: ast::LiteralKind::Int(BigInt::from(2))
                                })
                            }
                        ),
                    )
                }
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

#[test]
fn parse_precedence() {
    let result = parse_text("1 + 2 + 3");
    if let Ok(program) = result {
        assert_eq!(program.statements.len(), 1);
    } else {
        assert!(false, "Program failed to parse");
    }
}

#[test]
fn parse_func() {
    let source = "\
add (x, y, z) -> 
    x + y + z

add(1, 2, 3)
";
    if let Err(err) = parse_text(source) {
        assert!(false, "Function def failed to parse: {:?}", err);
    }
}
