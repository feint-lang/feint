pub type ExecuteResult = Result<VMState, String>;

#[derive(Debug)]
pub struct ExecuteError {
    pub error: ExecuteErrorKind,
}

impl ExecuteError {
    pub fn new(error: ExecuteErrorKind) -> Self {
        Self { error }
    }
}

#[derive(Debug)]
pub enum ExecuteErrorKind {
    GenericError(String),
}

#[derive(Debug, PartialEq)]
pub enum VMState {
    Idle,
    Halted(i32),
}
