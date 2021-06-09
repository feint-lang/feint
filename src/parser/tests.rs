use num_bigint::BigInt;

use crate::ast;
use crate::parser::*;
use crate::util::BinaryOperator;

#[test]
fn parse_empty() {
    let result = parse_text("", true);
    if let Ok(program) = result {
        assert_eq!(program.statements.len(), 0);
    } else {
        assert!(false, "Program failed to parse: {:?}", result);
    }
}

#[test]
    #[rustfmt::skip]
    fn parse_int() {
        let result = parse_text("1", true);
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
fn parse_simple_assignment() {
    //      R
    //      |
    //      n=
    //      |
    //      1
    let result = parse_text("n = 1", true);
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
        let result = parse_text("1 + 2", true);
        assert!(result.is_ok());
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
                                    kind: ast::ExprKind::Literal(
                                        ast::Literal {
                                            kind: ast::LiteralKind::Int(BigInt::from(1))
                                        }
                                    )
                                }
                            ),
                            // +
                            BinaryOperator::Add,
                            Box::new(
                                // 2
                                ast::Expr {
                                    kind: ast::ExprKind::Literal(
                                        ast::Literal {
                                            kind: ast::LiteralKind::Int(BigInt::from(2))
                                        }
                                    )
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
    let result = parse_text("n = 1 + 2", true);
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
    let result = parse_text("a = 1\nb = a + 2\n", true);
    if let Ok(program) = result {
        assert_eq!(program.statements.len(), 2);
        // TODO: More checks
    } else {
        assert!(false, "Program failed to parse");
    }
}

#[test]
fn parse_precedence() {
    let result = parse_text("1 + 2 + 3", true);
    if let Ok(program) = result {
        assert_eq!(program.statements.len(), 1);
    } else {
        assert!(false, "Program failed to parse");
    }
}

#[test]
fn parse_func() {
    let source = "\
func (x, y) -> 
    x + y

func(1, 2)
";
    let result = parse_text(source, true);
    if let Ok(program) = result {
    } else {
        assert!(false, "Function def failed to parse: {:?}", result.unwrap_err());
    }
}
