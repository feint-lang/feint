use crate::compiler::CompErrKind;
use crate::parser::ParseErrKind;
use crate::scanner::ScanErrKind;
use crate::vm::{RuntimeErrKind, VMState};
use core::fmt;
use std::fmt::Formatter;

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

    /// Return exit code if this error wraps a runtime exit error.
    pub fn exit_code(&self) -> Option<u8> {
        if let ExeErrKind::RuntimeErr(RuntimeErrKind::Exit(code)) = self.kind {
            Some(code)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum ExeErrKind {
    Bootstrap(String),
    ModuleNotFound(String, Option<String>),
    CouldNotReadSourceFile(String),
    ScanErr(ScanErrKind),
    ParseErr(ParseErrKind),
    CompErr(CompErrKind),
    RuntimeErr(RuntimeErrKind),
    ReplErr(String),
}

impl fmt::Display for ExeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl fmt::Display for ExeErrKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ExeErrKind::*;
        let msg = match self {
            Bootstrap(msg) => format!("Bootstrap process failed: {msg}"),
            ModuleNotFound(name, maybe_path) => format!("{name} @ {maybe_path:?}"),
            CouldNotReadSourceFile(file_name) => {
                format!("Could not read source file: {file_name}")
            }
            ScanErr(kind) => format!("Scan error: {kind:?}"),
            ParseErr(kind) => format!("Parse error: {kind:?}"),
            CompErr(kind) => format!("Compilation error: {kind:?}"),
            RuntimeErr(kind) => format!("Runtime error: {kind:?}"),
            ReplErr(msg) => format!("REPL error: {msg}"),
        };
        write!(f, "{msg}")
    }
}
