use std::fmt;
use std::fmt::Formatter;

use crate::compiler::CompErr;
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

    pub fn new_object_not_found_err(index: usize) -> Self {
        Self::new(RuntimeErrKind::ObjectNotFound(index))
    }

    pub fn new_name_err<S: Into<String>>(message: S) -> Self {
        Self::new(RuntimeErrKind::NameErr(message.into()))
    }

    pub fn new_type_err<S: Into<String>>(message: S) -> Self {
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
    ExpectedVar(String),
    ParseErr(ParseErr),
    CompErr(CompErr),
    UnhandledInstruction(String),
    AttributeDoesNotExist(String),
    AttributeCannotBeSet(String),
    TypeErr(String),
    NameErr(String),
    StringFormatErr(String),
    NotCallable(ObjectRef),
}

impl fmt::Display for RuntimeErrKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
