#[derive(Debug, PartialEq)]
pub enum Token {
    LeftParen,           // (
    RightParen,          // )
    LeftSquareBracket,   // [
    RightSquareBracket,  // ]
    LeftAngleBracket,    // <
    RightAngleBracket,   // >

    BlockStart,  // No symbol (implied by newline in specific cases)
    BlockEnd,    // No symbol (implied by newline in specific cases)

    // Fundamental types
    True,   // true
    False,  // false
    Int(String),
    Float(String),
    String(String),

    // Single-character symbols
    Equal,   // =
    Star,    // *
    Slash,   // /
    Plus,    // +
    Minus,   // -
    Not,     // !
    Dot,     // .

    // Multi-character Symbols
    EqualEqual,          // ==
    DoubleStar,          // **
    MulEqual,            // *=
    DivEqual,            // /=
    PlusEqual,           // +=
    MinusEqual,          // -=
    NotEqual,            // !=
    GreaterThanOrEqual,  // >=
    LessThanOrEqual,     // <=
    LoopFeed,            // <-
    Range,               // ..

    Comment(String),                  // # ... (to end of line)
    TypeIdentifier(String),           // Name
    TypeMethodIdentifier(String),     // @name (called via type)
    SpecialMethodIdentifier(String),  // $name (e.g., $bool on a type)
    Identifier(String),               // name

    Unknown(char),
    Eof,
}
