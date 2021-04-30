#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    LeftParen,          // (
    RightParen,         // )
    LeftSquareBracket,  // [
    RightSquareBracket, // ]
    LeftAngleBracket,   // <
    RightAngleBracket,  // >

    Comma, // ,

    // Fundamental types
    Int(String),    // 1
    Float(String),  // 1.0
    String(String), // "1"

    // Single-character operators
    Equal, // =
    Star,  // *
    Slash, // /
    Plus,  // +
    Minus, // -
    Not,   // !
    Dot,   // .

    // Multi-character operators
    EqualEqual,         // ==
    And,                // &&
    Or,                 // ||
    DoubleStar,         // **
    MulEqual,           // *=
    DivEqual,           // /=
    PlusEqual,          // +=
    MinusEqual,         // -=
    NotEqual,           // !=
    GreaterThanOrEqual, // >=
    LessThanOrEqual,    // <=
    LoopFeed,           // <-
    Range,              // ..
    AsBool,             // !! (the boolean evaluation of an object)

    // This indicates the start of a block.
    // For functions and methods it can also be followed by a return
    // type.
    ReturnType, // ->

    // Identifiers
    Identifier(String),              // name
    TypeIdentifier(String),          // Name
    TypeMethodIdentifier(String),    // @name (called via type)
    SpecialMethodIdentifier(String), // $name (e.g., $bool on a type)

    Comment(String), // # ... (to end of line)
    Unknown(char),
    NeedsMoreInput(String),
    EndOfInput,
}
