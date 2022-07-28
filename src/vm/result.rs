use std::fmt;
use std::fmt::Formatter;

use crate::compiler::CompErr;
use crate::parser::ParseErr;
use crate::types::ObjectRef;

pub type ExeResult = Result<VMState, RuntimeErr>;
pub type RuntimeResult = Result<ObjectRef, RuntimeErr>;
pub type RuntimeBoolResult = Result<bool, RuntimeErr>;
pub type PopObjResult = Result<ObjectRef, RuntimeErr>;
pub type PopNObjResult = Result<Vec<ObjectRef>, RuntimeErr>;
pub type PeekObjResult = Result<Option<ObjectRef>, RuntimeErr>;

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

    pub fn new_attr_does_not_exist<S: Into<String>>(type_name: S, name: S) -> Self {
        Self::new(RuntimeErrKind::AttrDoesNotExist(type_name.into(), name.into()))
    }

    pub fn new_attr_cannot_be_set<S: Into<String>>(name: S) -> Self {
        Self::new(RuntimeErrKind::AttrCannotBeSet(name.into()))
    }

    pub fn new_item_does_not_exit<S: Into<String>>(name: S) -> Self {
        Self::new(RuntimeErrKind::ItemDoesNotExist(name.into()))
    }

    pub fn new_item_cannot_be_set<S: Into<String>>(name: S) -> Self {
        Self::new(RuntimeErrKind::ItemCannotBeSet(name.into()))
    }

    pub fn new_index_out_of_bounds(index: usize) -> Self {
        Self::new(RuntimeErrKind::IndexOutOfBounds(index))
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
    NotEnoughValuesOnStack(usize),
    ObjectNotFound(usize),
    ExpectedVar(String),
    ParseErr(ParseErr),
    CompErr(CompErr),
    UnhandledInstruction(String),
    TypeErr(String),
    NameErr(String),
    StringFormatErr(String),
    AttrDoesNotExist(String, String),
    AttrCannotBeSet(String),
    ItemDoesNotExist(String),
    ItemCannotBeSet(String),
    IndexOutOfBounds(usize),
    NotCallable(ObjectRef),

    // Move?
    CouldNotReadFile(String),
}

impl fmt::Display for RuntimeErrKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
