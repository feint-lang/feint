use crate::ast::Program;
use crate::scanner::{ScanError, TokenWithLocation};

pub type ParseResult = Result<Program, ParseError>;

#[derive(Clone, Debug)]
pub struct ParseError {
    pub kind: ParseErrorKind,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind) -> Self {
        Self { kind }
    }
}

#[derive(Clone, Debug)]
pub enum ParseErrorKind {
    ScanError(ScanError),
    CouldNotOpenSourceFile(String),
    UnhandledToken(TokenWithLocation),
    ExpectedExpression(TokenWithLocation),
}
