use std::fmt;

use num_bigint::BigInt;

use crate::util::Location;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    LeftParen,          // (
    RightParen,         // )
    LeftSquareBracket,  // [
    RightSquareBracket, // ]
    LeftAngleBracket,   // <
    RightAngleBracket,  // >

    Colon, // :
    Comma, // ,

    // Fundamental types
    Float(f64),           // 1.0, 1.0E+10
    Int(BigInt),          // 1, 1_000, 0b1, 0o1, ox1 (digits, radix)
    String(String),       // "words words words"
    FormatString(String), // $"words {name_in_scope} words"

    // Single-character operators
    Caret,     // ^
    Star,      // *
    Slash,     // /
    Percent,   // %
    Plus,      // +
    Minus,     // -
    Bang,      // !
    Dot,       // .
    Ampersand, // &
    Pipe,      // |
    Equal,     // =

    // Multi-character operators
    EqualEqual,         // ==
    EqualEqualEqual,    // === (use instead of is???)
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

    // Indicates start of function or block
    FuncStart, // ->

    ScopeStart, // Start of nested scope
    ScopeEnd,   // End of nested scope

    // Keywords
    Nil,           // nil
    True,          // true
    False,         // false
    Import,        // import <module>
    From,          // import from <module>: x, y, z
    Package,       // import from package.<module>: x, y, z
    As,            // import <module> as <name>
    Is,            // Identity (use === instead?)
    Let,           // let (???)
    Block,         // block
    If,            // if
    ElseIf,        // elif
    Else,          // else
    Loop,          // ??? (while true, like Rust)
    For,           // ??? or use <-
    While,         // ??? or use <-
    Break,         // break
    Continue,      // continue
    Jump,          // jump label
    Label(String), // label:
    Print,         // print (TEMP)

    // Identifiers
    Ident(String),              // name
    TypeIdent(String),          // Name
    TypeMethodIdent(String),    // @name (called via type)
    SpecialMethodIdent(String), // $name (e.g., $bool, $str)

    EndOfStatement,
    EndOfInput,
}

impl Token {
    pub fn as_str(&self) -> &str {
        match self {
            Self::LeftParen => "(",
            Self::RightParen => ")",

            Self::Colon => ":",
            Self::Comma => ",",

            Self::Caret => "^",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Percent => "%",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Bang => "!",
            Self::Dot => ".",
            Self::Equal => "=",

            Self::DoubleSlash => "//",
            Self::BangBang => "!!",
            Self::EqualEqual => "==",
            Self::EqualEqualEqual => "===",
            Self::NotEqual => "!=",
            Self::And => "&&",
            Self::Or => "||",

            Self::FuncStart => "->",
            Self::ScopeStart => "start of block",
            Self::ScopeEnd => "end of block",

            // Keywords
            Self::Block => "block",
            Self::Jump => "jump",
            Self::Label(_name) => "label",

            // Identifiers
            Self::Ident(s)
            | Self::TypeIdent(s)
            | Self::TypeMethodIdent(s)
            | Self::SpecialMethodIdent(s) => s.as_str(),

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
