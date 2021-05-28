use std::num::ParseFloatError;

use num_bigint::ParseBigIntError;

use crate::util::Location;

use super::TokenWithLocation;

pub type ScanResult = Result<TokenWithLocation, ScanError>;

#[derive(Clone, Debug)]
pub struct ScanError {
    pub kind: ScanErrorKind,
    pub location: Location,
}

impl ScanError {
    pub fn new(kind: ScanErrorKind, location: Location) -> Self {
        Self { kind, location }
    }
}

#[derive(Clone, Debug)]
pub enum ScanErrorKind {
    InvalidIndent(u8), // Indent is not a multiple of 4 (number of spaces)
    UnexpectedIndent(u8), // Indent in unexpected place (indent level)
    WhitespaceAfterIndent, // Non-space whitespace after indent
    UnexpectedWhitespace, // Other unexpected whitespace
    UnterminatedString(String), // String with no closing quote
    UnexpectedCharacter(char), // Char not recognized as token or start of token
    UnmatchedOpeningBracket(char), // Closing bracket with no matching opening bracket
    UnmatchedClosingBracket(char), // Opening bracket with no matching closing bracket
    ParseIntError(ParseBigIntError),
    ParseFloatError(ParseFloatError),
    CouldNotOpenSourceFile(String, String), // path, reason
    TooMuchWhitespace,
    ExpectedNewline,
}
