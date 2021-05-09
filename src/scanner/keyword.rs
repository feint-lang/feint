pub enum Keyword {
    // Booleans
    True,
    False,

    // Imports
    Import,  // import <module>
    From,    // import from <module>: x, y, z
    Package, // import from package.<module>: x, y, z
    As,      // import <module> as <name>

    // Declarations
    Let, // ???

    // Anonymous block/scope
    Block,

    // Conditionals
    If,
    Else,
    ElseIf,

    // Loops
    For,   // ??? or use <-
    Loop,  // ??? (while true, like Rust)
    While, // ??? or use <-
}
