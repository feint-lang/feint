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

#[derive(Debug)]
pub enum VMState {
    Idle(Option<ObjectRef>),
    Halted(u8),
}

#[derive(Clone, Debug)]
pub enum ValueStackKind {
    GlobalConstant(ObjectRef, usize),
    Constant(ObjectRef, usize),
    Var(ObjectRef, usize, String),
    CellVar(ObjectRef, usize, String),
    Temp(ObjectRef),
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

    pub fn exit(code: u8) -> Self {
        Self::new(RuntimeErrKind::Exit(code))
    }

    pub fn assertion_failed(message: String) -> Self {
        Self::new(RuntimeErrKind::AssertionFailed(message))
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

    pub fn stack_index_out_of_bounds(index: usize) -> Self {
        Self::new(RuntimeErrKind::StackIndexOutOfBounds(index))
    }

    pub fn frame_index_out_of_bounds(index: usize) -> Self {
        Self::new(RuntimeErrKind::FrameIndexOutOfBounds(index))
    }

    pub fn recursion_depth_exceeded(max_call_depth: CallDepth) -> Self {
        Self::new(RuntimeErrKind::RecursionDepthExceeded(max_call_depth))
    }

    pub fn constant_not_found(index: usize) -> Self {
        Self::new(RuntimeErrKind::ConstantNotFound(index))
    }

    pub fn captured_var_not_found<S: Into<String>>(name: S) -> Self {
        Self::new(RuntimeErrKind::CapturedVarNotFound(name.into()))
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

    pub fn index_out_of_bounds<S: Into<String>>(type_name: S, index: usize) -> Self {
        Self::new(RuntimeErrKind::IndexOutOfBounds(type_name.into(), index))
    }

    pub fn not_callable<S: Into<String>>(type_name: S) -> Self {
        Self::new(RuntimeErrKind::NotCallable(type_name.into()))
    }

    pub fn arg_err<S: Into<String>>(message: S) -> Self {
        Self::new(RuntimeErrKind::ArgErr(message.into()))
    }
}

impl fmt::Display for RuntimeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

#[derive(Clone, Debug)]
pub enum RuntimeErrKind {
    Exit(u8),
    AssertionFailed(String),
    EmptyStack,
    NotEnoughValuesOnStack(usize),
    EmptyCallStack,
    StackIndexOutOfBounds(usize),
    FrameIndexOutOfBounds(usize),
    RecursionDepthExceeded(CallDepth),
    ConstantNotFound(usize),
    CapturedVarNotFound(String),
    ExpectedVar(String),
    ParseErr(ParseErr),
    CompErr(CompErr),
    UnhandledInstruction(String),
    TypeErr(String),
    NameErr(String),
    StringFormatErr(String),
    IndexOutOfBounds(String, usize),
    NotCallable(String),
    ArgErr(String),
}

impl fmt::Display for RuntimeErrKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
