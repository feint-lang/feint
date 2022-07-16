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
    Halted(u8),
}

// Runtime errors ------------------------------------------------------

#[derive(Clone, Debug)]
pub struct RuntimeErr {
    pub kind: RuntimeErrKind,
}

impl RuntimeErr {
    pub fn new(kind: RuntimeErrKind) -> Self {
        Self { kind }
    }

    pub fn new_type_error<S: Into<String>>(message: S) -> Self {
        Self::new(RuntimeErrKind::TypeErr(message.into()))
    }
}

impl fmt::Display for RuntimeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

#[derive(Clone, Debug)]
pub enum RuntimeErrKind {
    EmptyStack,
    ObjectNotFound(usize),
    NotEnoughValuesOnStack(String),
    ParseErr(ParseErr),
    CompilationErr(CompilationErr),
    UnhandledInstruction(String),
    AttributeDoesNotExist(String),
    AttributeCannotBeSet(String),
    TypeErr(String),
    NameErr(String),
    StringFormatErr(String),
}

impl fmt::Display for RuntimeErrKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
