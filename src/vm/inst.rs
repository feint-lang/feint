use crate::util::{
    BinaryOperator, CompareOperator, InplaceOperator, Location, UnaryCompareOperator,
    UnaryOperator,
};

#[derive(Debug, PartialEq)]
pub enum Inst {
    DisplayStack(String),

    NoOp,

    Pop,

    // Global constants are shared globally by all code units.
    LoadGlobalConst(usize),

    // Special global constants with a known index.
    LoadNil,        // 0
    LoadTrue,       // 1
    LoadFalse,      // 2
    LoadEmptyTuple, // 3

    ScopeStart,
    ScopeEnd,

    StatementStart(Location, Location),

    // Other constants are local to a given code unit.
    LoadConst(usize),

    DeclareVar(String),
    AssignVar(String),
    LoadVar(String),
    LoadOuterVar(String),

    // These are analogous to AssignVar and LoadVar. Assignment wraps
    // the value in a cell so that it can be shared. Loading unwraps the
    // value.
    AssignCell(String),
    LoadCell(String),

    // Load captured value to TOS (a special case of LoadCell).
    LoadCaptured(String),

    // Jumps -----------------------------------------------------------
    //
    // For all jump instructions, the first arg is the target address
    // relative to the jump address. The second arg is a flag to
    // indicate a forward or reverse jump. The third arg is the scope
    // exit count.
    //
    // Relative addresses allow instructions to be inserted BEFORE any
    // forward jumps or AFTER any backward jumps within a code segment.
    // Mainly this is to allow instructions to be inserted at the
    // beginning of functions.

    // Jump unconditionally.
    Jump(usize, bool, usize),

    // Jump unconditionally and push nil onto stack.
    JumpPushNil(usize, bool, usize),

    // If top of stack is false, jump to address. Otherwise, continue.
    JumpIfNot(usize, bool, usize),

    UnaryOp(UnaryOperator),
    UnaryCompareOp(UnaryCompareOperator),

    BinaryOp(BinaryOperator),
    CompareOp(CompareOperator),
    InplaceOp(InplaceOperator),

    // Call function with N values from top of stack. The args are
    // ordered such that the 1st arg is at TOS and other args are below
    // it.
    Call(usize),

    // Return is a jump target that exits the function's scope.
    Return,

    // These make compound objects from the top N items on the stack.
    MakeString(usize),
    MakeTuple(usize),
    MakeList(usize),
    MakeMap(usize),

    // Capture set for function--a list of names for the function to
    // capture. If empty, a regular function will be created.
    CaptureSet(Vec<String>),

    // Make function or closure depending on capture set.
    MakeFunc(usize),

    LoadModule(String),

    Placeholder(usize, Box<Inst>, String), // address, instruction, error message
    FreeVarPlaceholder(usize, String),     // address, var name
    BreakPlaceholder(usize, usize),        // jump address, scope depth
    ContinuePlaceholder(usize, usize),     // jump address, scope depth

    // NOTE: This is used for explicit return statements. It will be
    //       replaced with a jump to a RETURN target.
    ReturnPlaceholder(usize, usize), // jump address, scope depth

    Halt(u8),
    HaltTop,
}
