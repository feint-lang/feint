use num_bigint::BigInt;

use crate::ast::*;
use crate::util::Location;

#[test]
#[rustfmt::skip]
fn create_ast() {
    let program = Program::new(vec![
        // 1 + 2
        Statement::new_expr(
            Expr::new_binary_op(
                Expr::new_literal(
                    Literal::new_int(BigInt::from(1)),
                    Location::new(1, 1),
                    Location::new(1, 1),
                ),
                "+",
                Expr::new_literal(
                    Literal::new_int(BigInt::from(2)),
                    Location::new(1, 5),
                    Location::new(1, 5),
                ),
                Location::new(1, 1),
                Location::new(1, 5),
            ),
            Location::new(1, 1),
            Location::new(1, 5),
        ),
        // 1 - 1
        Statement::new_expr(
            Expr::new_binary_op(
                Expr::new_literal(
                    Literal::new_int(BigInt::from(1)),
                    Location::new(2, 1),
                    Location::new(2, 1),
                ),
                "-",
                Expr::new_literal(
                    Literal::new_int(BigInt::from(1)),
                    Location::new(2, 1),
                    Location::new(2, 1),
                ),
                Location::new(2, 1),
                Location::new(2, 1),
            ),
            Location::new(2, 1),
            Location::new(2, 5),
        )
    ]);
    eprintln!("{:?}", program);
}
