use std::io::Error;
use std::num::ParseFloatError;

use num_bigint::ParseBigIntError;

use crate::util::Location;

use super::TokenWithLocation;

pub type ScanResult = Result<TokenWithLocation, ScanError>;

#[derive(Debug)]
pub struct ScanError {
    pub error: ScanErrorKind,
    pub location: Location,
}

impl ScanError {
    pub fn new(error: ScanErrorKind, location: Location) -> Self {
        Self { error, location }
    }
}

#[derive(Debug)]
pub enum ScanErrorKind {
    InvalidIndent(i32), // Indent is not a multiple of 4 (number of spaces)
    UnexpectedIndent(i32), // Indent in unexpected place (indent level)
    WhitespaceAfterIndent, // Non-space whitespace after indent
    UnexpectedWhitespace, // Other unexpected whitespace
    UnterminatedString(String), // String with no closing quote
    UnexpectedCharacter(char), // Char not recognized as token or start of token
    UnmatchedOpeningBracket(char), // Closing bracket with no matching opening bracket
    UnmatchedClosingBracket(char), // Opening bracket with no matching closing bracket
    ParseIntError(ParseBigIntError),
    ParseFloatError(ParseFloatError),
    CouldNotOpenSourceFile(Error),
}
