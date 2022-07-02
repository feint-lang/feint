use crate::vm::Chunk;

pub type CompilationResult = Result<Chunk, CompilationErr>;

#[derive(Clone, Debug)]
pub struct CompilationErr {
    pub kind: CompilationErrKind,
}

impl CompilationErr {
    pub fn new(kind: CompilationErrKind) -> Self {
        Self { kind }
    }
}

#[derive(Clone, Debug)]
pub enum CompilationErrKind {
    VisitErr(String),
}
