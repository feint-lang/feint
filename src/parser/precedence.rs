use crate::scanner::Token;

/// Get unary precedence of token.
pub fn get_unary_precedence(token: &Token) -> u8 {
    get_operator_precedence(token).0
}

/// Get binary precedence of token.
pub fn get_binary_precedence(token: &Token) -> u8 {
    get_operator_precedence(token).1
}

/// Return true if the token represents a right-associate operator.
pub fn is_right_associative(token: &Token) -> bool {
    // Exponentiation and assignment
    matches!(token, Token::Caret | Token::Equal)
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
    use Token::*;
    match token {
        Equal                            // a = b
        | MulEqual                       // a *= b
        | DivEqual                       // a /= b
        | PlusEqual                      // a -= b
        | MinusEqual         => (0, 1),  // a += b
        
        | Or                 => (0, 2),  // a || b
        | And                => (0, 3),  // a && b

        | EqualEqualEqual                // a === b     (is)
        | NotEqualEqual                  // a !== b     (is not)
        | EqualEqual                     // a == b
        | NotEqual                       // a != b
        | LessThan                       // a < b
        | LessThanOrEqual                // a <= b
        | GreaterThan                    // a > b
        | GreaterThanOrEqual => (0, 4),  // a >= b
        
        | Plus                           // +a, a + b
        | Minus              => (8, 5),  // -a, a - b
        
        | Star                           // a * b
        | Slash                          // a / b       (floating point div)
        | DoubleSlash                    // a // b      (floor div)
        | Percent            => (0, 6),  // a % b
       
        | Caret              => (0, 7),  // a ^ b       (exponentiation)

        | BangBang                       // !!a         (as bool)
        | Bang               => (8, 0),  // !a          (logical not)

        | LParen             => (0, 9),  // x(...)      (call)
        | Dot                => (0, 10), // x.y
        
        _                    => (0, 0),  // not an operator
    }
}
