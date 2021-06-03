use crate::util::{BinaryOperator, UnaryOperator};

pub type Instructions = Vec<Instruction>;

#[derive(Debug)]
pub enum Instruction {
    NoOp,
    Push(usize),
    Pop,
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
    BlockEnd(usize),
    Print, // Print value at top of stack
    Return,
    Halt(i32),
}
