use std::fmt;
use std::fmt::Formatter;

use crate::compiler::CompilationErr;
use crate::parser::ParseErr;
use crate::types::ObjectRef;

pub type ExeResult = Result<VMState, RuntimeErr>;
pub type RuntimeResult = Result<ObjectRef, RuntimeErr>;
pub type RuntimeBoolResult = Result<bool, RuntimeErr>;

#[derive(Debug, PartialEq)]
pub enum VMState {
    Idle,
    Halted(i32),
}

// Runtime errors ------------------------------------------------------

#[derive(Debug)]
pub struct RuntimeErr {
    pub kind: RuntimeErrKind,
}

impl RuntimeErr {
    pub fn new(kind: RuntimeErrKind) -> Self {
        Self { kind }
    }

    pub fn new_type_error<S: Into<String>>(message: S) -> Self {
        Self::new(RuntimeErrKind::TypeError(message.into()))
    }
}

impl fmt::Display for RuntimeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

#[derive(Debug)]
pub enum RuntimeErrKind {
    EmptyStack,
    NotEnoughValuesOnStack(String),
    ParseError(ParseErr),
    CompilationError(CompilationErr),
    UnhandledInstruction(String),
    AttributeDoesNotExist(String),
    AttributeCannotBeSet(String),
    TypeError(String),
    NameError(String),
}

impl fmt::Display for RuntimeErrKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
