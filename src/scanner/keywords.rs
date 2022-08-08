use std::collections::HashMap;

use once_cell::sync::Lazy;

use super::{Token, Token::*};

/// Map of keywords to their respective Tokens.
pub static KEYWORDS: Lazy<HashMap<&'static str, Token>> = Lazy::new(|| {
    [
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
        ("return", Return),
        ("jump", Jump),
    ]
    .iter()
    .cloned()
    .collect()
});
