use super::{Location, TokenWithLocation};

pub type ScanResult = Result<TokenWithLocation, ScanError>;

#[derive(Debug)]
pub struct ScanError {
    pub error: ScanErrorType,
    pub location: Location,
}

impl ScanError {
    pub fn new(error: ScanErrorType, location: Location) -> Self {
        Self { error, location }
    }
}

#[derive(Debug)]
pub enum ScanErrorType {
    InvalidIndent(i32),    // Indent is not a multiple of 4
    UnexpectedIndent(i32), // Indent in unexpected place
    WhitespaceAfterIndent, // Non-space whitespace after indent
    UnexpectedWhitespace,  // Other unexpected whitespace
    UnterminatedString(String),
    UnknownToken(char),
    UnmatchedOpeningBracket(char),
    UnmatchedClosingBracket(char),
}
