use std::io;

use crate::ast::Program;
use crate::scanner;
use crate::util::Location;

pub type ParseResult = Result<Program, ParseError>;

#[derive(Debug)]
pub struct ParseError {
    pub kind: ParseErrorKind,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind) -> Self {
        Self { kind }
    }
}

#[derive(Debug)]
pub enum ParseErrorKind {
    ScanError(scanner::ScanError),
    CouldNotOpenSourceFile(io::Error),
    UnhandledToken(scanner::TokenWithLocation),
}
