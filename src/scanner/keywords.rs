use std::collections::HashMap;

use lazy_static::lazy_static;

use super::{Token, Token::*};

lazy_static! {
    /// Map of keywords to their respective Tokens.
    pub static ref KEYWORDS: HashMap<&'static str, Token> = [
        ("nil", Nil),
        ("true", True),
        ("false", False),
        ("import", Import),
        ("export", Export),
        ("from", From),
        ("package", Package),
        ("as", As),
        ("block", Block),
        ("if", If),
        ("else", Else),
        ("match", Match),
        ("loop", Loop),
        ("break", Break),
        ("continue", Continue),
        ("jump", Jump),
    ]
    .iter()
    .cloned()
    .collect();
}
