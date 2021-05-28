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
    match token {
        Token::Caret => true, // a ^ b (exponentiation)
        Token::Equal => true, // a = b = c (assignment)
        _ => false,
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
pub fn get_operator_precedence(token: &Token) -> (u8, u8) {
    match token {
        | Token::Equal =>      (0, 1), // a = b
        
        | Token::EqualEqual => (0, 2), // a == b
        
        | Token::Plus                  // +a, a + b
        | Token::Minus =>      (6, 3), // -a, a - b
        
        | Token::Star                  // a * b
        | Token::Slash                 // a / b   (floating point div)
        | Token::DoubleSlash           // a // b  (floor div)
        | Token::Percent =>    (0, 4), // a % b
       
        | Token::Caret =>      (0, 5), // a ^ b   (exponentiation)
        
        | Token::Bang =>       (6, 0), // !a      (logical not)
        
        // Not an operator
        _ =>                   (0, 0),
    }
}
