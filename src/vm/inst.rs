use crate::util::{
    BinaryOperator, CompareOperator, InplaceOperator, Location, UnaryCompareOperator,
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

    ScopeStart,
    ScopeEnd,

    StatementStart(Location, Location),

    // Other constants are local to a given code unit.
    LoadConst(usize),

    // Store value at top of stack into local--pop top of stack and
    // replace local value lower in stack with top of stack value.
    StoreLocal(usize),

    // Load local value onto top of stack--retrieve (copy) local value
    // from slot lower in stack and push it onto TOS.
    LoadLocal(usize),

    DeclareVar(String),
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
    Return,      // Return is a no-op jump target

    // These make compound objects from the top N items on the stack.
    MakeString(usize),
    MakeTuple(usize),
    MakeList(usize),
    MakeMap(usize),

    // Make function closure for constant.
    MakeClosure(usize),

    LoadModule(String),

    Placeholder(usize, Box<Inst>, String),
    BreakPlaceholder(usize, usize), // jump address, scope depth
    ContinuePlaceholder(usize, usize), // jump address, scope depth

    // NOTE: This is used for explicit return statements. It will be
    //       replaced with a jump to a RETURN target.
    ReturnPlaceholder(usize, usize), // jump address, scope depth

    Halt(u8),
    HaltTop,
}
