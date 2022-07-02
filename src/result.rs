use crate::compiler::CompilationErrKind;
use crate::parser::ParseErrKind;
use crate::scanner::ScanErrKind;
use crate::vm::{RuntimeErrKind, VMState};

/// Result type used by top level runners.
///
/// On success, Ok(None) or OK(Some(String)) should be returned. In both
/// cases, the program will exit with error code 0. In the latter case,
/// the specified message will be printed to stdout just before exiting.
///
/// On error, Err((u8, Option<(String))) should be returned. Note
/// that on error, a message is *always* required. The program will
/// print the specified message to stderr and then exit with the
/// specified error code.
pub(crate) type ExitResult = Result<Option<String>, (u8, Option<String>)>;

/// Result type used by top level program executor.
pub(crate) type ExeResult = Result<VMState, ExeErr>;

#[derive(Debug)]
pub struct ExeErr {
    pub kind: ExeErrKind,
}

impl ExeErr {
    pub fn new(kind: ExeErrKind) -> Self {
        Self { kind }
    }
}

#[derive(Debug)]
pub enum ExeErrKind {
    CouldNotReadSourceFileErr(String),
    ScanErr(ScanErrKind),
    ParseErr(ParseErrKind),
    CompilationErr(CompilationErrKind),
    RuntimeErr(RuntimeErrKind),
}
