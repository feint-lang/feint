use crate::util::{BinaryOperator, UnaryOperator};

pub type Chunk = Vec<Inst>;

#[derive(Debug, PartialEq)]
pub enum Inst {
    NoOp,

    Push(usize),
    Pop,
    LoadConst(usize),

    ScopeStart,
    ScopeEnd(usize),

    DeclareVar(String),
    AssignVar(String),
    LoadVar(String),

    // Jumps -----------------------------------------------------------
    //
    // NOTE: For all jump instructions, the last arg is the scope exit
    //       count.

    // Jump unconditionally
    Jump(usize, usize), // address

    // If top of stack is true, jump to address. Otherwise, continue.
    JumpIf(usize, usize),

    // If top of stack is false, jump to address. Otherwise, continue.
    JumpIfNot(usize, usize),

    // If top of stack is true, jump to first address. Otherwise,
    // jump to second address.
    JumpIfElse(usize, usize, usize),

    UnaryOp(UnaryOperator),
    BinaryOp(BinaryOperator),

    Call(usize), // Call function with N values from top of stack
    Return,

    // These make compound objects from the top N items on the stack.
    MakeString(usize),
    MakeTuple(usize),

    Placeholder(usize, Box<Inst>, String),
    BreakPlaceholder(usize, usize), // address, scope depth
    ContinuePlaceholder(usize, usize), // address, scope depth

    Halt(u8),
}
