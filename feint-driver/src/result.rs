use core::fmt;
use std::fmt::Formatter;

use feint_compiler::{CompErrKind, ParseErrKind, ScanErrKind};
use feint_vm::{RuntimeErrKind, VMState};

pub type CallDepth = usize;

/// Result type used by top level program driver.
pub type DriverResult = Result<VMState, DriverErr>;

#[derive(Debug)]
pub struct DriverErr {
    pub kind: DriverErrKind,
}

impl DriverErr {
    pub fn new(kind: DriverErrKind) -> Self {
        Self { kind }
    }

    /// Return exit code if this error wraps a runtime exit error.
    pub fn exit_code(&self) -> Option<u8> {
        if let DriverErrKind::RuntimeErr(RuntimeErrKind::Exit(code)) = self.kind {
            Some(code)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum DriverErrKind {
    Bootstrap(String),
    ModuleDirNotFound(String),
    ModuleNotFound(String),
    CouldNotReadSourceFile(String),
    ScanErr(ScanErrKind),
    ParseErr(ParseErrKind),
    CompErr(CompErrKind),
    RuntimeErr(RuntimeErrKind),
    ReplErr(String),
}

impl fmt::Display for DriverErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl fmt::Display for DriverErrKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use DriverErrKind::*;
        let msg = match self {
            Bootstrap(msg) => format!("Bootstrap process failed: {msg}"),
            ModuleDirNotFound(path) => format!(
                concat!(
                    "Module directory not found: {}\n",
                    "Please double check your module search path."
                ),
                path
            ),
            ModuleNotFound(name) => {
                format!("Module not found: {name}")
            }
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
