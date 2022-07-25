use std::fmt;

use crate::format::FormatStrToken;
use num_bigint::BigInt;

use crate::util::Location;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    LParen,   // (
    RParen,   // )
    LBracket, // [
    RBracket, // ]

    Colon, // :
    Comma, // ,

    // Fundamental types
    Float(f64),                     // 1.0, 1.0E+10
    Int(BigInt),                    // 1, 1_000, 0b1, 0o1, ox1 (digits, radix)
    Str(String),                    // "words words words"
    FormatStr(Vec<FormatStrToken>), // $"words {name_in_scope} words"

    // Single-character operators
    Caret,       // ^
    Star,        // *
    Slash,       // /
    Percent,     // %
    Plus,        // +
    Minus,       // -
    Bang,        // !
    Dot,         // .
    Ampersand,   // &
    Pipe,        // |
    LessThan,    // <
    GreaterThan, // >
    Equal,       // =

    // Multi-character operators
    EqualEqual,         // ==
    EqualEqualEqual,    // ===
    And,                // &&
    Or,                 // ||
    DoubleStar,         // **
    DoubleSlash,        // //
    NotEqual,           // !=
    GreaterThanOrEqual, // >=
    LessThanOrEqual,    // <=
    LoopFeed,           // <-
    Range,              // ..
    RangeInclusive,     // ...
    BangBang,           // !! (the boolean evaluation of an object)

    // In-place operators
    // TODO: If reassignment isn't allowed, these don't make sense
    MulEqual,   // *=
    DivEqual,   // /=
    PlusEqual,  // +=
    MinusEqual, // -=

    ScopeStart, // -> (start of scope: function, block, etc)
    ScopeEnd,   // end of scope (implicit, no symbol)

    InlineScopeStart, // -> (start of inline scope: function, block, etc)
    InlineScopeEnd,   // end of inline scope (implicit, no symbol)

    // Keywords
    Nil,           // nil
    True,          // true
    False,         // false
    Import,        // import <module>
    From,          // import from <module>: x, y, z
    Package,       // import from package.<module>: x, y, z
    Export,        // export <object>
    As,            // import <module> as <name>
    Let,           // let (???)
    Block,         // block
    If,            // if
    Else,          // else
    Match,         // match
    Loop,          // ??? (while true, like Rust)
    Break,         // break
    Continue,      // continue
    Jump,          // jump label
    Label(String), // label:

    // Identifiers
    Ident(String),            // name
    TypeIdent(String),        // Name
    TypeMethIdent(String),    // @name (called via type)
    SpecialMethIdent(String), // $name (e.g., $bool, $str)

    EndOfStatement,
    EndOfInput,
}

impl Token {
    pub fn as_str(&self) -> &str {
        match self {
            Self::LParen => "(",
            Self::RParen => ")",
            Self::LBracket => "[",
            Self::RBracket => "]",

            Self::Colon => ":",
            Self::Comma => ",",

            Self::Caret => "^",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Percent => "%",
            Self::Plus => "+",
            Self::PlusEqual => "+=",
            Self::Minus => "-",
            Self::MinusEqual => "-=",
            Self::Bang => "!",
            Self::Dot => ".",
            Self::Equal => "=",

            Self::DoubleSlash => "//",
            Self::BangBang => "!!",
            Self::EqualEqual => "==",
            Self::EqualEqualEqual => "===",
            Self::NotEqual => "!=",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",
            Self::And => "&&",
            Self::Or => "||",

            Self::ScopeStart => "->",
            Self::ScopeEnd => "<scope end>",

            Self::InlineScopeStart => "-> (inline)",
            Self::InlineScopeEnd => "<inline scope end>",

            // Keywords
            Self::Block => "block",
            Self::If => "if",
            Self::Else => "else",
            Self::Match => "match",
            Self::Jump => "jump",
            Self::Label(_name) => "label",

            // Identifiers
            Self::Ident(s)
            | Self::TypeIdent(s)
            | Self::TypeMethIdent(s)
            | Self::SpecialMethIdent(s) => s.as_str(),

            Self::EndOfStatement => "end of statement",
            Self::EndOfInput => "EOI",

            _ => unimplemented!("{:?}.as_str()", self),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// A token with its start and end locations in the source.
#[derive(Clone, Debug, PartialEq)]
pub struct TokenWithLocation {
    pub token: Token,
    pub start: Location,
    pub end: Location,
}

impl TokenWithLocation {
    pub fn new(token: Token, start: Location, end: Location) -> Self {
        Self { token, start, end }
    }

    pub fn as_str(&self) -> &str {
        self.token.as_str()
    }
}

impl fmt::Display for TokenWithLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} -> {}", self.as_str(), self.start, self.end)
    }
}
