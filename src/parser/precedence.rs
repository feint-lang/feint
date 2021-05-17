use crate::scanner::Token;

/// Get unary precedence of token.
pub fn get_unary_precedence(token: &Token) -> u8 {
    get_operator_precedence(token).0
}

/// Get binary precedence of token.
pub fn get_binary_precedence(token: &Token) -> u8 {
    get_operator_precedence(token).1
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
pub fn get_operator_precedence(token: &Token) -> (u8, u8) {
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
