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
    LoadNil,   // 0
    LoadTrue,  // 1
    LoadFalse, // 2

    FuncScopeStart(usize),
    ScopeStart,
    ScopeEnd,

    StatementStart(Location, Location),

    // Other constants are local to a given code unit.
    LoadConst(usize),

    // Store value at TOS as local.
    StoreLocal(usize),

    // Load local value onto top of stack--retrieve (copy) local value
    // from slot lower in stack and push it onto TOS.
    LoadLocal(usize),

    StoreLocalAndCell(usize, usize), // local index, cell index

    // Load a captured var from the "heap" to TOS.
    LoadCell(usize),

    // Convert arg at offset from TOS to local for use as call arg.
    ToArg(usize),
    ToArgAndCell(usize, usize),

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

    // Make closure wrapping function.
    MakeClosure(
        usize,               // constant index of wrapped function
        Vec<(usize, usize)>, // (relative call stack index, local index within frame)+
    ),

    LoadModule(String),

    Placeholder(usize, Box<Inst>, String), // address, instruction, error message
    VarPlaceholder(usize, String),         // address, var name
    BreakPlaceholder(usize, usize),        // jump address, scope depth
    ContinuePlaceholder(usize, usize),     // jump address, scope depth

    // NOTE: This is used for explicit return statements. It will be
    //       replaced with a jump to a RETURN target.
    ReturnPlaceholder(usize, usize), // jump address, scope depth

    Halt(u8),
    HaltTop,
}
