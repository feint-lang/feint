use std::num::ParseFloatError;

use num_bigint::ParseBigIntError;

use crate::format::FormatStrErr;
use crate::util::Location;

use super::{Token, TokenWithLocation};

pub type AddTokenResult = Result<Token, ScanErr>;
pub type AddTokensResult = Result<(), ScanErr>;
pub type ScanTokenResult = Result<TokenWithLocation, ScanErr>;
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
    ExpectedBlock,     // Block expected but not provided
    ExpectedIndentedBlock(u8), // Expected an indented block
    UnterminatedStr(String), // String with no closing quote
    UnexpectedChar(char), // Char not recognized as token or start of token
    UnmatchedOpeningBracket(char), // Closing bracket with no matching opening bracket
    UnmatchedClosingBracket(char), // Opening bracket with no matching closing bracket
    ParseIntErr(ParseBigIntError),
    ParseFloatErr(ParseFloatError),
    FormatStrErr(FormatStrErr),
    TooMuchWhitespace,
}
