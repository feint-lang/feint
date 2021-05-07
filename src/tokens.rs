use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Indent(u8), // Space characters following a newline

    LeftParen,          // (
    RightParen,         // )
    LeftSquareBracket,  // [
    RightSquareBracket, // ]
    LeftAngleBracket,   // <
    RightAngleBracket,  // >

    Colon, // :
    Comma, // ,

    // Fundamental types
    True,
    False,
    Float(String),              // 1.0
    Int(String),                // 1
    String(String),             // "1" (does NOT include quotes)
    UnterminatedString(String), // "1 (DOES include opening quote)

    // Single-character operators
    Equal,     // =
    Star,      // *
    Slash,     // /
    Plus,      // +
    Minus,     // -
    Not,       // !
    Dot,       // .
    Percent,   // %
    Caret,     // ^
    Ampersand, // &
    Pipe,      // |

    // Multi-character operators
    EqualEqual,         // ==
    And,                // &&
    Or,                 // ||
    DoubleStar,         // **
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

    // Indicates start of function or block
    FuncStart, // ->

    // Identifiers
    Identifier(String),              // name
    TypeIdentifier(String),          // Name
    TypeMethodIdentifier(String),    // @name (called via type)
    SpecialMethodIdentifier(String), // $name (e.g., $bool on a type)

    Comment(String), // # ... (to end of line)

    UnexpectedWhitespace,
    Unknown(char),
    EndOfInput,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TokenWithPosition {
    pub token: Token,
    pub line_no: usize,
    pub col_no: usize,
}

impl TokenWithPosition {
    pub fn new(token: Token, line_no: usize, col_no: usize) -> Self {
        TokenWithPosition {
            token,
            line_no,
            col_no,
        }
    }
}

impl fmt::Display for TokenWithPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token {}:{} {:?}", self.line_no, self.col_no, self.token)
    }
}
