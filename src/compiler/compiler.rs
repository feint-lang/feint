use crate::ast;
use crate::vm::{format_instructions, Instruction, Instructions};

use super::result::CompilationResult;

pub fn compile(program: ast::Program, debug: bool) -> CompilationResult {
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
