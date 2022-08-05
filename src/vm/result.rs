use std::fmt;
use std::fmt::Formatter;

use crate::compiler::CompErr;
use crate::parser::ParseErr;
use crate::types::ObjectRef;

pub type CallDepth = usize;
pub type ExeResult = Result<VMState, RuntimeErr>;
pub type RuntimeResult = Result<(), RuntimeErr>;
pub type RuntimeObjResult = Result<ObjectRef, RuntimeErr>;
pub type RuntimeBoolResult = Result<bool, RuntimeErr>;
pub type PopResult = Result<ValueStackKind, RuntimeErr>;
pub type PopNResult = Result<Vec<ValueStackKind>, RuntimeErr>;
pub type PopObjResult = Result<ObjectRef, RuntimeErr>;
pub type PopNObjResult = Result<Vec<ObjectRef>, RuntimeErr>;
pub type PeekResult<'a> = Result<&'a ValueStackKind, RuntimeErr>;
pub type PeekObjResult = Result<ObjectRef, RuntimeErr>;

#[derive(Debug, PartialEq)]
pub enum VMState {
    Idle,
    Halted(u8),
}

#[derive(Clone, Debug)]
pub enum ValueStackKind {
    GlobalConstant(ObjectRef, usize),
    Constant(ObjectRef, usize),
    Var(ObjectRef, usize, String),
    Local(ObjectRef, usize),
    Temp(ObjectRef),
    ReturnVal(ObjectRef),
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

    pub fn new_empty_statck() -> Self {
        Self::new(RuntimeErrKind::EmptyStack)
    }

    pub fn new_not_enough_values_on_stack(n: usize) -> Self {
        Self::new(RuntimeErrKind::NotEnoughValuesOnStack(n))
    }

    pub fn new_recursion_depth_exceeded(max_call_depth: CallDepth) -> Self {
        Self::new(RuntimeErrKind::RecursionDepthExceeded(max_call_depth))
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

    pub fn new_attr_cannot_be_set<S: Into<String>>(type_name: S, name: S) -> Self {
        Self::new(RuntimeErrKind::AttrCannotBeSet(type_name.into(), name.into()))
    }

    pub fn new_item_does_not_exist<S: Into<String>>(
        type_name: S,
        index: usize,
    ) -> Self {
        Self::new(RuntimeErrKind::ItemDoesNotExist(type_name.into(), index))
    }

    pub fn new_item_cannot_be_set<S: Into<String>>(type_name: S, index: usize) -> Self {
        Self::new(RuntimeErrKind::ItemCannotBeSet(type_name.into(), index))
    }

    pub fn new_index_out_of_bounds<S: Into<String>>(
        type_name: S,
        index: usize,
    ) -> Self {
        Self::new(RuntimeErrKind::IndexOutOfBounds(type_name.into(), index))
    }

    pub fn new_not_callable<S: Into<String>>(type_name: S) -> Self {
        Self::new(RuntimeErrKind::NotCallable(type_name.into()))
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
    RecursionDepthExceeded(CallDepth),
    ObjectNotFound(usize),
    ExpectedVar(String),
    ParseErr(ParseErr),
    CompErr(CompErr),
    UnhandledInstruction(String),
    TypeErr(String),
    NameErr(String),
    StringFormatErr(String),
    AttrDoesNotExist(String, String),
    AttrCannotBeSet(String, String),
    ItemDoesNotExist(String, usize),
    ItemCannotBeSet(String, usize),
    IndexOutOfBounds(String, usize),
    NotCallable(String),
    EmptyCallStack,

    // Move?
    CouldNotReadFile(String),
}

impl fmt::Display for RuntimeErrKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
