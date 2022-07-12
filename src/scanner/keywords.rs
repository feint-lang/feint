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
        ("let", Let),         // ???
        ("block", Block),
        ("if", If),
        ("else", Else),
        ("match", Match),
        ("loop", Loop),
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
