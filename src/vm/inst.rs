use crate::util::{
    BinaryOperator, CompareOperator, InplaceOperator, UnaryCompareOperator,
    UnaryOperator,
};

#[derive(Debug, PartialEq)]
pub enum Inst {
    NoOp,
    Pop,

    // Global constants are shared globally by all code units.
    LoadGlobalConst(usize),

    // Special global constants with a known index.
    LoadNil,   // 0
    LoadTrue,  // 1
    LoadFalse, // 2

    // Other constants are local to a given code unit.
    LoadConst(usize),

    ScopeStart,
    ScopeEnd,

    DeclareVar(String),
    AssignLocal(usize, String),
    AssignVar(String),
    LoadVar(String),

    // Jumps -----------------------------------------------------------
    //
    // NOTE: For all jump instructions, the last arg is the scope exit
    //       count.

    // Jump unconditionally
    Jump(usize, usize), // address

    // Jump unconditionally and push nil onto stack
    JumpPushNil(usize, usize), // address

    // If top of stack is true, jump to address. Otherwise, continue.
    JumpIf(usize, usize),

    // If top of stack is false, jump to address. Otherwise, continue.
    JumpIfNot(usize, usize),

    // If top of stack is true, jump to first address. Otherwise,
    // jump to second address.
    JumpIfElse(usize, usize, usize),

    UnaryOp(UnaryOperator),
    UnaryCompareOp(UnaryCompareOperator),

    BinaryOp(BinaryOperator),
    CompareOp(CompareOperator),
    InplaceOp(InplaceOperator),

    Call(usize), // Call function with N values from top of stack
    Return,

    // These make compound objects from the top N items on the stack.
    MakeString(usize),
    MakeTuple(usize),

    Placeholder(usize, Box<Inst>, String),
    BreakPlaceholder(usize, usize), // address, scope depth
    ContinuePlaceholder(usize, usize), // address, scope depth

    Halt(u8),
    HaltTop,
}
