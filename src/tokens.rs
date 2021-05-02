#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Whitespace(String), // Contiguous sequence of whitespace characters

    LeftParen,          // (
    RightParen,         // )
    LeftSquareBracket,  // [
    RightSquareBracket, // ]
    LeftAngleBracket,   // <
    RightAngleBracket,  // >

    Comma, // ,

    // Fundamental types
    Int(String),                // 1
    Float(String),              // 1.0
    String(String),             // "1" (does NOT include quotes)
    UnterminatedString(String), // "1 (DOES include opening quote)

    // Single-character operators
    Equal,   // =
    Star,    // *
    Slash,   // /
    Plus,    // +
    Minus,   // -
    Not,     // !
    Dot,     // .
    Percent, // %
    Caret,   // ^

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
    AsBool,             // !! (the boolean evaluation of an object)

    // In-place operators
    // TODO: If reassignment isn't allowed, these don't make sense
    MulEqual,   // *=
    DivEqual,   // /=
    PlusEqual,  // +=
    MinusEqual, // -=

    // Can be used for named functions or anonymous blocks
    BlockStart, // ->

    // Identifiers
    Identifier(String),              // name
    TypeIdentifier(String),          // Name
    TypeMethodIdentifier(String),    // @name (called via type)
    SpecialMethodIdentifier(String), // $name (e.g., $bool on a type)

    Comment(String), // # ... (to end of line)
    Unknown(char),
    EndOfInput,
}
