use std::num::ParseFloatError;

use num_bigint::ParseBigIntError;

use crate::format::FormatStringErr;
use crate::util::Location;

use super::TokenWithLocation;

pub type ScanResult = Result<TokenWithLocation, ScanErr>;
pub type ScanTokensResult = Result<Vec<TokenWithLocation>, ScanErr>;

#[derive(Clone, Debug)]
pub struct ScanErr {
    pub kind: ScanErrKind,
    pub location: Location,
}

impl ScanErr {
    pub fn new(kind: ScanErrKind, location: Location) -> Self {
        Self { kind, location }
    }
}

#[derive(Clone, Debug)]
pub enum ScanErrKind {
    InvalidIndent(u8), // Indent is not a multiple of 4 (number of spaces)
    UnexpectedIndent(u8), // Indent in unexpected place (indent level)
    WhitespaceAfterIndent, // Non-space whitespace after indent
    UnexpectedWhitespace, // Other unexpected whitespace
    UnterminatedString(String), // String with no closing quote
    UnexpectedCharacter(char), // Char not recognized as token or start of token
    UnmatchedOpeningBracket(char), // Closing bracket with no matching opening bracket
    UnmatchedClosingBracket(char), // Opening bracket with no matching closing bracket
    ParseIntErr(ParseBigIntError),
    ParseFloatErr(ParseFloatError),
    FormatStringErr(FormatStringErr),
    TooMuchWhitespace,
}
