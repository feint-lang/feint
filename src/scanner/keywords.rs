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
        ("from", From),
        ("package", Package),
        ("as", As),
        ("is", Is),           // ???
        ("let", Let),         // ???
        ("block", Block),
        ("if", If),
        ("elif", ElseIf),
        ("else", Else),
        ("loop", Loop),       // ???
        ("for", For),         // ???
        ("while", While),     // ???
        ("break", Break),
        ("continue", Continue),
        ("jump", Jump),       // goto label
        ("print", Print),
    ]
    .iter()
    .cloned()
    .collect();
}
