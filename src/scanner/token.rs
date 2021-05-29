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
    Equal,     // =
    Star,      // *
    Slash,     // /
    Plus,      // +
    Minus,     // -
    Bang,      // !
    Dot,       // .
    Percent,   // %
    Caret,     // ^
    Ampersand, // &
    Pipe,      // |

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
    AsBool,             // !! (the boolean evaluation of an object)

    // In-place operators
    // TODO: If reassignment isn't allowed, these don't make sense
    MulEqual,   // *=
    DivEqual,   // /=
    PlusEqual,  // +=
    MinusEqual, // -=

    // Indicates start of function or block/scope
    FuncStart, // ->

    BlockStart, // Start of indented block
    BlockEnd,   // End of indented block

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
            Self::Caret => "^",
            Self::Star => "*",
            Self::Slash => "/",
            Self::DoubleSlash => "//",
            Self::Percent => "%",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Bang => "!",
            Self::AsBool => "!!",
            Self::EqualEqual => "==",
            Self::NotEqual => "!=",
            Self::And => "&&",
            Self::Or => "||",
            Self::Equal => "=",
            Self::Jump => "jump",
            Self::Label(name) => "label",
            Self::Ident(s)
            | Self::TypeIdent(s)
            | Self::TypeMethodIdent(s)
            | Self::SpecialMethodIdent(s) => s.as_str(),
            Self::EndOfInput => "EOI",
            _ => panic!("{:?} (need to implement Token.as_str() for {})", self, self),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
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
}

impl fmt::Display for TokenWithLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} -> {}", self.token, self.start, self.end)
    }
}
