use std::fmt;
use std::fmt::Formatter;

use crate::compiler::CompErr;
use crate::parser::ParseErr;
use crate::types::ObjectRef;

pub type CallDepth = usize;
pub type VMExeResult = Result<VMState, RuntimeErr>;
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
    TempLocal(ObjectRef, usize),
    ReturnVal(ObjectRef),
}

// Runtime errors ------------------------------------------------------

#[derive(Clone, Debug)]
pub struct RuntimeErr {
    pub kind: RuntimeErrKind,
}

impl RuntimeErr {
    fn new(kind: RuntimeErrKind) -> Self {
        Self { kind }
    }

    pub fn empty_stack() -> Self {
        Self::new(RuntimeErrKind::EmptyStack)
    }

    pub fn not_enough_values_on_stack(n: usize) -> Self {
        Self::new(RuntimeErrKind::NotEnoughValuesOnStack(n))
    }

    pub fn empty_call_stack() -> Self {
        Self::new(RuntimeErrKind::EmptyCallStack)
    }

    pub fn frame_index_out_of_bounds(index: usize) -> Self {
        Self::new(RuntimeErrKind::FrameIndexOutOfBounds(index))
    }

    pub fn recursion_depth_exceeded(max_call_depth: CallDepth) -> Self {
        Self::new(RuntimeErrKind::RecursionDepthExceeded(max_call_depth))
    }

    pub fn object_not_found_err(index: usize) -> Self {
        Self::new(RuntimeErrKind::ObjectNotFound(index))
    }

    pub fn expected_var<S: Into<String>>(message: S) -> Self {
        Self::new(RuntimeErrKind::ExpectedVar(message.into()))
    }

    pub fn name_err<S: Into<String>>(message: S) -> Self {
        Self::new(RuntimeErrKind::NameErr(message.into()))
    }

    pub fn type_err<S: Into<String>>(message: S) -> Self {
        Self::new(RuntimeErrKind::TypeErr(message.into()))
    }

    pub fn attr_does_not_exist<S: Into<String>>(type_name: S, name: S) -> Self {
        Self::new(RuntimeErrKind::AttrDoesNotExist(type_name.into(), name.into()))
    }

    pub fn attr_cannot_be_set<S: Into<String>>(type_name: S, name: S) -> Self {
        Self::new(RuntimeErrKind::AttrCannotBeSet(type_name.into(), name.into()))
    }

    pub fn item_does_not_exist<S: Into<String>>(type_name: S, index: usize) -> Self {
        Self::new(RuntimeErrKind::ItemDoesNotExist(type_name.into(), index))
    }

    pub fn item_cannot_be_set<S: Into<String>>(type_name: S, index: usize) -> Self {
        Self::new(RuntimeErrKind::ItemCannotBeSet(type_name.into(), index))
    }

    pub fn index_out_of_bounds<S: Into<String>>(type_name: S, index: usize) -> Self {
        Self::new(RuntimeErrKind::IndexOutOfBounds(type_name.into(), index))
    }

    pub fn not_callable<S: Into<String>>(type_name: S) -> Self {
        Self::new(RuntimeErrKind::NotCallable(type_name.into()))
    }

    pub fn arg_err<S: Into<String>>(message: S) -> Self {
        Self::new(RuntimeErrKind::ArgErr(message.into()))
    }

    pub fn could_not_read_file<S: Into<String>>(message: S) -> Self {
        Self::new(RuntimeErrKind::CouldNotReadFile(message.into()))
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
    EmptyCallStack,
    FrameIndexOutOfBounds(usize),
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

    ArgErr(String),
    CouldNotReadFile(String),
}

impl fmt::Display for RuntimeErrKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
