use crate::util::{
    BinaryOperator, CompareOperator, InplaceOperator, Location, UnaryCompareOperator,
    UnaryOperator,
};

/// NOTE: When adding or removing instructions, the PartialEq impl
///       below must also be updated.
#[derive(Debug)]
pub enum Inst {
    NoOp,

    // Pop TOS and discard it.
    Pop,

    // Global constants are shared globally by all code units.
    LoadGlobalConst(usize),

    // Special global constants with a known index.
    LoadNil,        // 0
    LoadTrue,       // 1
    LoadFalse,      // 2
    LoadAlways,     // 3
    LoadEmptyStr,   // 4
    LoadEmptyTuple, // 5

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

    // RETURN is a jump target at the end of a function. Its only
    // purpose is to serve as a jump target for explicit returns.
    Return,

    // THROW provides a simplistic way to exit and print an error. It
    // can be used as a synonym for system.exit() by throwing an Int
    // that can be interpreted as an exit code (a u8).
    Throw,

    // These make compound objects from the top N items on the stack.
    MakeString(usize),
    MakeTuple(usize),
    MakeList(usize),
    MakeMap(usize),

    // Capture set for function--a list of names for the function to
    // capture. If empty, a regular function will be created.
    CaptureSet(Vec<String>),

    // Make function or closure depending on capture set. MAKE_FUNC
    // expects the following entries at TOS:
    //
    // TOS capture_set: Map (added by CAPTURE_SET)
    //     func: Func       (added by LOAD_CONST)
    MakeFunc,

    LoadModule(String),

    Halt(u8),
    HaltTop,

    // Placeholders ----------------------------------------------------
    //
    // Placeholders are inserted during compilation and later updated.
    // All placeholders must be replaced or a runtime error will be
    // thrown.
    Placeholder(usize, Box<Inst>, String), // address, instruction, error message
    FreeVarPlaceholder(usize, String),     // address, var name
    BreakPlaceholder(usize, usize),        // jump address, scope depth
    ContinuePlaceholder(usize, usize),     // jump address, scope depth

    // NOTE: This is used for explicit return statements. It will be
    //       replaced with a jump to a RETURN target.
    ReturnPlaceholder(usize, usize), // jump address, scope depth

    // Miscellaneous ---------------------------------------------------

    // Pop TOS and print it.
    PrintTop,

    DisplayStack(String),
}

impl PartialEq for Inst {
    fn eq(&self, other: &Self) -> bool {
        use Inst::*;

        match (self, other) {
            (NoOp, NoOp) => true,
            (Pop, Pop) => true,
            (LoadGlobalConst(a), LoadGlobalConst(b)) => a == b,
            (LoadNil, LoadNil) => true,
            (LoadTrue, LoadTrue) => true,
            (LoadFalse, LoadFalse) => true,
            (LoadAlways, LoadAlways) => true,
            (LoadEmptyStr, LoadEmptyStr) => true,
            (LoadEmptyTuple, LoadEmptyTuple) => true,
            (ScopeStart, ScopeStart) => true,
            (ScopeEnd, ScopeEnd) => true,
            (StatementStart(..), StatementStart(..)) => true,
            (LoadConst(a), LoadConst(b)) => a == b,
            (DeclareVar(a), DeclareVar(b)) => a == b,
            (AssignVar(a), AssignVar(b)) => a == b,
            (LoadVar(a), LoadVar(b)) => a == b,
            (LoadOuterVar(a), LoadOuterVar(b)) => a == b,
            (AssignCell(a), AssignCell(b)) => a == b,
            (LoadCell(a), LoadCell(b)) => a == b,
            (LoadCaptured(a), LoadCaptured(b)) => a == b,
            (Jump(a, b, c), Jump(d, e, f)) => (a, b, c) == (d, e, f),
            (JumpPushNil(a, b, c), JumpPushNil(d, e, f)) => (a, b, c) == (d, e, f),
            (JumpIfNot(a, b, c), JumpIfNot(d, e, f)) => (a, b, c) == (d, e, f),
            (UnaryOp(a), UnaryOp(b)) => a == b,
            (UnaryCompareOp(a), UnaryCompareOp(b)) => a == b,
            (BinaryOp(a), BinaryOp(b)) => a == b,
            (CompareOp(a), CompareOp(b)) => a == b,
            (InplaceOp(a), InplaceOp(b)) => a == b,
            (Call(a), Call(b)) => a == b,
            (Return, Return) => true,
            (MakeString(a), MakeString(b)) => a == b,
            (MakeTuple(a), MakeTuple(b)) => a == b,
            (MakeList(a), MakeList(b)) => a == b,
            (MakeMap(a), MakeMap(b)) => a == b,
            (CaptureSet(a), CaptureSet(b)) => a == b,
            (MakeFunc, MakeFunc) => true,
            (LoadModule(a), LoadModule(b)) => a == b,
            (Halt(a), Halt(b)) => a == b,
            (HaltTop, HaltTop) => true,
            _ => false,
        }
    }
}
