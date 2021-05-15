use num_bigint::BigInt;

use crate::ast::*;

#[test]
#[rustfmt::skip]
fn create_ast() {
    let program = Program::new(vec![
        // 1 + 2
        Statement::new_expression(
            Expression::new_binary_operation(
                "+",
                Expression::new_literal(
                    Literal::new_int(BigInt::from(1))
                ),
                Expression::new_literal(
                    Literal::new_int(BigInt::from(2))
                ),
            )
        ),
        // 1 - 1
        Statement::new_expression(
            Expression::new_binary_operation(
                "-",
                Expression::new_literal(
                    Literal::new_int(BigInt::from(1))
                ),
                Expression::new_literal(
                    Literal::new_int(BigInt::from(1))
                ),
            )
        )
    ]);
    println!("{:?}", program);
}
