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

#[derive(Debug)]
pub enum ExecutionErrorKind {
    GenericError(String),
}

#[derive(Debug, PartialEq)]
pub enum VMState {
    Idle,
    Halted(i32),
}
