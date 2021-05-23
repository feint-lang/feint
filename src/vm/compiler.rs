use crate::ast;

use super::instruction::{format_instructions, Instruction, Instructions};

pub fn compile(program: ast::Program, debug: bool) -> CompileResult {
    // TODO: Walk AST and emit instructions
    if debug {
        eprintln!("COMPILING:\n{:?}", program);
    }
    let mut instructions: Instructions = vec![];
    instructions.push(Instruction::Halt(0));
    if debug {
        eprintln!("INSTRUCTIONS:\n{}", format_instructions(&instructions));
    }
    Ok(instructions)
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
