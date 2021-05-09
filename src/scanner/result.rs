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
    UnexpectedIndent(u8), // number of spaces
    UnexpectedWhitespace, // i.e, after an indent
    UnterminatedString(String),
    UnknownToken(char),
    UnmatchedOpeningBracket(char),
    UnmatchedClosingBracket(char),
}
