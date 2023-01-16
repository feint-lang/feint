use num_bigint::BigInt;

use crate::ast::*;
use crate::scanner::Token;
use crate::source::Location;

#[test]
#[rustfmt::skip]
fn create_ast() {
    let program = Module::new(vec![
        // 1 + 2
        Statement::new_expr(
            Expr::new_binary_op(
                Expr::new_int(
                    BigInt::from(1),
                    Location::new(1, 1),
                    Location::new(1, 1),
                ),
                &Token::Plus,
                Expr::new_int(
                    BigInt::from(2),
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
                Expr::new_int(
                    BigInt::from(1),
                    Location::new(2, 1),
                    Location::new(2, 1),
                ),
                &Token::Minus,
                Expr::new_int(
                    BigInt::from(1),
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
