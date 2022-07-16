use crate::util::{BinaryOperator, UnaryOperator};

pub type Chunk = Vec<Inst>;

#[derive(Debug, PartialEq)]
pub enum Inst {
    NoOp,
    Push(usize),
    Pop,

    // Jump unconditionally
    Jump(usize),

    // If top of stack is true, jump to address. Otherwise, continue.
    JumpIf(usize),

    // If top of stack is false, jump to address. Otherwise, continue.
    JumpIfNot(usize),

    // If top of stack is true, jump to first address. Otherwise,
    // jump to second address.
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
    Halt(u8),

    InternalErr(String),

    // These make compound objects from the top N items on the stack.
    MakeString(usize),
    MakeTuple(usize),
}
