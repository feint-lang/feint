use std::fmt;
use std::fmt::Formatter;

use crate::compiler::CompilationError;
use crate::parser::ParseError;
use crate::types::ObjectRef;
use crate::vm::Instruction;

pub type ExecutionResult = Result<VMState, RuntimeError>;
pub type RuntimeResult = Result<ObjectRef, RuntimeError>;

#[derive(Debug, PartialEq)]
pub enum VMState {
    Running,
    Idle,
    Halted(i32),
}

// Runtime errors ------------------------------------------------------

#[derive(Debug)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
}

impl RuntimeError {
    pub fn new(kind: RuntimeErrorKind) -> Self {
        Self { kind }
    }

    pub fn new_type_error<S: Into<String>>(message: S) -> Self {
        Self::new(RuntimeErrorKind::TypeError(message.into()))
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

#[derive(Debug)]
pub enum RuntimeErrorKind {
    ObjectStoreIndexError(usize),
    NotEnoughValuesOnStack(String),
    ParseError(ParseError),
    CompilationError(CompilationError),
    UnhandledInstruction(String),
    TypeError(String),
}

impl fmt::Display for RuntimeErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
