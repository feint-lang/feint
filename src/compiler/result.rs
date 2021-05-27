use crate::types::Builtins;
use crate::vm::{Instructions, ObjectStore};

pub type CompilationResult = Result<Instructions, CompilationError>;

#[derive(Clone, Debug)]
pub struct CompilationError {
    pub kind: CompilationErrorKind,
}

impl CompilationError {
    pub fn new(kind: CompilationErrorKind) -> Self {
        Self { kind }
    }
}

#[derive(Clone, Debug)]
pub enum CompilationErrorKind {
    VisitError(String),
}
