use crate::util::{BinaryOperator, UnaryOperator};

pub type Chunk = Vec<Inst>;

#[derive(Debug, PartialEq)]
pub enum Inst {
    NoOp,
    Push(usize),
    Pop,

    // Jump unconditionally
    Jump(usize),

    // If top of stack is true, jump to first address
    // Otherwise, jump to second address
    JumpIfElse(usize, usize),

    UnaryOp(UnaryOperator),
    BinaryOp(BinaryOperator),
    LoadConst(usize),
    DeclareVar(String),
    AssignVar(String),
    LoadVar(String),
    ScopeStart,
    ScopeEnd(usize),
    Print, // Print value at top of stack
    Return,
    Halt(i32),
}
