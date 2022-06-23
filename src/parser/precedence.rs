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
        Token::Comma                    // a, b
        | Token::Equal       => (0, 1), // a = b
        
        // TODO: This was added for use in ternary expressions, but
        //       it doesn't work with if/else blocks. Maybe just use
        //       `cond ? yes : no`?
        // | Token::If          => (0, 2), // if ...
        
        | Token::Or          => (0, 3), // a || b
        | Token::And         => (0, 4), // a && b

        | Token::Is                     // a == b
        | Token::EqualEqual             // a == b
        | Token::NotEqual    => (0, 5), // a != b
        
        | Token::Plus                   // +a, a + b
        | Token::Minus       => (9, 6), // -a, a - b
        
        | Token::Star                   // a * b
        | Token::Slash                  // a / b     (floating point div)
        | Token::DoubleSlash            // a // b    (floor div)
        | Token::Percent     => (0, 7), // a % b
       
        | Token::Caret       => (0, 8), // a ^ b     (exponentiation)

        | Token::BangBang               // !!a       (as bool)
        | Token::Bang        => (9, 0), // !a        (logical not)
        
        _                    => (0, 0), // not an operator
    }
}
