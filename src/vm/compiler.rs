use crate::ast;

use super::instruction::{Instruction, Instructions};

pub fn compile(program: ast::Program, debug: bool) -> CompileResult {
    // TODO: Walk AST and emit instructions
    Ok(vec![Instruction::Halt(0)])
}

pub type CompileResult = Result<Instructions, CompilationError>;

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
    GenericError(String),
}
