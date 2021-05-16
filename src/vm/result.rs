use std::fmt;
use std::fmt::Formatter;

pub type ExecutionResult = Result<VMState, ExecutionError>;

#[derive(Debug)]
pub struct ExecutionError {
    pub error: ExecutionErrorKind,
}

impl ExecutionError {
    pub fn new(error: ExecutionErrorKind) -> Self {
        Self { error }
    }
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.error)
    }
}

#[derive(Debug)]
pub enum ExecutionErrorKind {
    GenericError(String),
    NotEnoughValuesOnStack,
}

impl fmt::Display for ExecutionErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, PartialEq)]
pub enum VMState {
    Idle,
    Halted(i32),
}
