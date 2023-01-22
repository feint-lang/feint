use std::fmt;

use num_bigint::BigInt;

use crate::format::FormatStrToken;
use crate::source::Location;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    // Groupings -------------------------------------------------------
    LParen,   // (
    RParen,   // )
    LBracket, // [
    RBracket, // ]
    LBrace,   // {
    RBrace,   // }

    // Miscellaneous ---------------------------------------------------
    Colon,    // :
    DotDot,   // ..
    Ellipsis, // ...
    LoopFeed, // <-

    // Fundamental types -----------------------------------------------
    At,                             // @ (used to represent the singleton Always)
    Int(BigInt),                    // 1, 1_000, 0b1, 0o1, ox1 (digits, radix)
    Float(f64),                     // 1.0, 1.0E+10
    Str(String),                    // "words words words"
    FormatStr(Vec<FormatStrToken>), // $"words {name_in_scope} words"

    // Operators -------------------------------------------------------
    Bang,     // !
    BangBang, // !! (the boolean evaluation of an object)

    Caret,       // ^
    Star,        // *
    Slash,       // /
    DoubleSlash, // //
    Percent,     // %
    Plus,        // +
    Minus,       // -
    Pipe,        // |
    Ampersand,   // &

    Equal, // =
    Dot,   // .
    Comma, // ,

    DollarDollar,       // $$
    DollarNot,          // $!
    EqualEqualEqual,    // ===
    NotEqualEqual,      // !==
    EqualEqual,         // ==
    NotEqual,           // !=
    And,                // &&
    Or,                 // ||
    NilOr,              // ??
    LessThan,           // <
    LessThanOrEqual,    // <=
    GreaterThan,        // >
    GreaterThanOrEqual, // >=

    MulEqual,   // *=
    DivEqual,   // /=
    PlusEqual,  // +=
    MinusEqual, // -=

    // Scope ---------------------------------------------------
    ScopeStart,       // -> (start of scope: block, etc)
    ScopeEnd,         // end of scope (implicit, no symbol)
    InlineScopeStart, // -> (start of inline scope: block, etc)
    InlineScopeEnd,   // end of inline scope (implicit, no symbol)

    FuncScopeStart,       // => (start of func scope)
    FuncInlineScopeStart, // => (start of func inline scope)

    // Keywords ------------------------------------------------
    Nil,           // nil
    True,          // true
    False,         // false
    Block,         // block
    If,            // if
    Else,          // else
    Match,         // match
    Loop,          // ??? (while true, like Rust)
    Break,         // break
    Continue,      // continue
    Return,        // return
    Jump,          // jump label
    Label(String), // :label:
    Halt,
    Print,

    // Import/export ---------------------------------------------------
    Import,             // import <path>
    ImportPath(String), // import <path>
    From,               // import from <module>: x, y, z
    Package,            // import from package.<module>: x, y, z
    Export,             // export <object>
    As,                 // import <path> as <name>

    // Identifiers -----------------------------------------------------
    Ident(String),         // ident_name
    ConstIdent(String),    // CONST_NAME
    TypeIdent(String),     // TypeName
    TypeFuncIdent(String), // @type_func_name (called via type)
    SpecialIdent(String),  // $special_name (e.g., $bool, $str)

    EndOfStatement,
    EndOfInput,
}

impl Token {
    pub fn as_str(&self) -> &str {
        match self {
            // Groupings -----------------------------------------------
            Self::LParen => "(",
            Self::RParen => ")",
            Self::LBracket => "[",
            Self::RBracket => "]",
            Self::LBrace => "{",
            Self::RBrace => "}",

            // Miscellaneous ---------------------------------------------------
            Self::At => "@",
            Self::Colon => ":",
            Self::DotDot => "..",
            Self::Ellipsis => "...",
            Self::LoopFeed => "<-",

            // Fundamental types -----------------------------------------------
            Self::Int(_) => "Int",
            Self::Float(_) => "Float",
            Self::Str(_) => "Str",
            Self::FormatStr(_) => "$Str",

            // Operators -----------------------------------------------
            Self::Bang => "!",
            Self::BangBang => "!!",

            Self::Caret => "^",
            Self::Star => "*",
            Self::Slash => "/",
            Self::DoubleSlash => "//",
            Self::Percent => "%",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Pipe => "|",
            Self::Ampersand => "&",

            Self::Equal => "=",
            Self::Dot => ".",
            Self::Comma => ",",

            Self::DollarDollar => "$$",
            Self::DollarNot => "$",
            Self::EqualEqualEqual => "===",
            Self::NotEqualEqual => "!==",
            Self::EqualEqual => "==",
            Self::NotEqual => "!=",
            Self::And => "&&",
            Self::Or => "||",
            Self::NilOr => "??",
            Self::LessThan => "<",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqual => ">=",

            Self::MulEqual => "*=",
            Self::DivEqual => "/=",
            Self::PlusEqual => "+=",
            Self::MinusEqual => "-=",

            // Scope ---------------------------------------------------
            Self::ScopeStart => "->",
            Self::ScopeEnd => "<scope end>",
            Self::InlineScopeStart => "-> (inline)",
            Self::InlineScopeEnd => "<inline scope end>",

            Self::FuncScopeStart => "=>",
            Self::FuncInlineScopeStart => "=> (inline)",

            // Keywords ------------------------------------------------
            Self::Nil => "nil",
            Self::True => "true",
            Self::False => "false",
            Self::Block => "block",
            Self::If => "if",
            Self::Else => "else",
            Self::Match => "match",
            Self::Loop => "loop",
            Self::Break => "break",
            Self::Continue => "continue",
            Self::Return => "return",
            Self::Jump => "jump",
            Self::Label(_name) => "label",
            Self::Halt => "$halt",
            Self::Print => "$print",

            // Import/export ---------------------------------------------------
            Self::Import => "import",
            Self::ImportPath(path) => path.as_str(),
            Self::From => "from",
            Self::Package => "package",
            Self::Export => "export",
            Self::As => "as",

            // Identifiers ---------------------------------------------
            Self::Ident(s)
            | Self::ConstIdent(s)
            | Self::TypeIdent(s)
            | Self::TypeFuncIdent(s)
            | Self::SpecialIdent(s) => s.as_str(),

            Self::EndOfStatement => "end of statement",
            Self::EndOfInput => "EOI",
        }
    }

    pub fn is_ident(&self) -> bool {
        matches!(
            self,
            Token::Ident(_)
                | Token::ConstIdent(_)
                | Token::TypeIdent(_)
                | Token::TypeFuncIdent(_)
                | Token::SpecialIdent(_)
        )
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
        write!(
            f,
            "{:?}({}) @ {} -> {})",
            self.token,
            self.as_str(),
            self.start,
            self.end
        )
    }
}
