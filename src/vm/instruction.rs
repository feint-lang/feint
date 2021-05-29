use std::fmt;

use crate::util::{BinaryOperator, UnaryOperator};

pub type Instructions = Vec<Instruction>;

#[derive(Debug)]
pub enum Instruction {
    Push(usize),
    Pop,
    StoreLabel(String),
    JumpToLabel(String),
    Jump(usize),        // Jump unconditionally
    JumpIfTrue(usize),  // Jump if top of stack is true
    JumpIfFalse(usize), // Jump if top of stack is false
    UnaryOp(UnaryOperator),
    BinaryOp(BinaryOperator),
    LoadConst(usize),
    DeclareVar(String),
    AssignVar(String),
    LoadVar(String),
    BlockStart,
    BlockEnd,
    Print, // Print value at top of stack
    Return,
    Halt(i32),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::Print => format!("PRINT"),
            Self::LoadConst(v) => format_aligned("LOAD_CONST", v),
            Self::UnaryOp(operator) => format_aligned("UNARY_OP", operator),
            Self::BinaryOp(operator) => format_aligned("BINARY_OP", operator),
            Self::Return => format!("RETURN"),
            i => format!("{:?}", i),
        };
        write!(f, "{}", string)
    }
}

fn format_aligned<T: fmt::Display>(name: &str, value: T) -> String {
    format!("{: <w$}{: <x$}", name, value, w = 16, x = 4)
}

/// Return a nicely formatted string of instructions.
///
/// NOTE: We can't directly implement Display for Vec<Instruction> and
///       creating a wrapper type just for that purpose seems like a
///       pain.
pub fn format_instructions(instructions: &Instructions) -> String {
    let mut offset = 0;
    instructions
        .iter()
        .map(|instruction| {
            let result =
                format!("{:0>offset_width$} {}", offset, instruction, offset_width = 4);
            offset += 1;
            result
        })
        .collect::<Vec<String>>()
        .join("\n")
}
