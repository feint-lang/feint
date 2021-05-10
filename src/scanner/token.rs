use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    EndOfInput,

    BlockStart,
    BlockEnd,

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
    Float(String),    // 1.0
    Int(String, u32), // 1, 0b1, 0o1, ox1 (digits, radix)
    String(String),   // "1" (does NOT include quotes)

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

    pub fn new_from_line_col(
        token: Token,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Self {
        Self::new(
            token,
            Location::new(start_line, start_col),
            Location::new(end_line, end_col),
        )
    }
}

impl fmt::Display for TokenWithLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token {} -> {} {:?}", self.start, self.end, self.token)
    }
}

// Represents a line and column in the source.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Location {
    pub line: usize,
    pub col: usize,
}

impl Location {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}
