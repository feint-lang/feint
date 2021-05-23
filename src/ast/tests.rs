use num_bigint::BigInt;

use crate::ast::*;

#[test]
#[rustfmt::skip]
fn create_ast() {
    let program = Program::new(vec![
        // 1 + 2
        Statement::new_expr(
            Expr::new_binary_operation(
                Expr::new_literal(
                    Literal::new_int(BigInt::from(1))
                ),
                "+",
                Expr::new_literal(
                    Literal::new_int(BigInt::from(2))
                ),
            )
        ),
        // 1 - 1
        Statement::new_expr(
            Expr::new_binary_operation(
                Expr::new_literal(
                    Literal::new_int(BigInt::from(1))
                ),
                "-",
                Expr::new_literal(
                    Literal::new_int(BigInt::from(1))
                ),
            )
        )
    ]);
    println!("{:?}", program);
}
