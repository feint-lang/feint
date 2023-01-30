use std::ops::Index;
use std::slice::Iter;

use feint_util::op::{BinaryOperator, CompareOperator, InplaceOperator, UnaryOperator};
use feint_util::source::Location;
use feint_util::string::format_doc;

use crate::types::{new, FuncTrait, ObjectRef};

type FreeVarEntry = (
    usize,    // address
    String,   // name
    Location, // source start
    Location, // source end
);

/// Code for a module or function.
#[derive(Debug)]
pub struct Code {
    chunk: Vec<Inst>,
    constants: Vec<ObjectRef>,
    // Vars defined outside of this unit of code.
    free_vars: Vec<FreeVarEntry>,
}

impl Default for Code {
    fn default() -> Self {
        Code::new(vec![], vec![], vec![])
    }
}

impl Index<usize> for Code {
    type Output = Inst;

    fn index(&self, index: usize) -> &Self::Output {
        &self.chunk[index]
    }
}

impl PartialEq for Code {
    fn eq(&self, other: &Self) -> bool {
        if self.chunk != other.chunk {
            return false;
        }
        if self.constants.len() != other.constants.len() {
            return false;
        }
        if self.free_vars != other.free_vars {
            return false;
        }
        for (c, d) in self.constants.iter().zip(other.constants.iter()) {
            let c = c.read().unwrap();
            let d = d.read().unwrap();
            if !c.is_equal(&*d) {
                return false;
            }
        }
        true
    }
}

impl Code {
    pub fn new(
        chunk: Vec<Inst>,
        constants: Vec<ObjectRef>,
        free_vars: Vec<FreeVarEntry>,
    ) -> Self {
        Self { chunk, constants, free_vars }
    }

    /// Initialize code object with a list of instructions, also known
    /// as a chunk.
    pub fn with_chunk(chunk: Vec<Inst>) -> Self {
        Self::new(chunk, vec![], vec![])
    }

    /// Extend this `Code` object with another `Code` object:
    ///
    /// - Extend instructions, adjusting constant indexes
    /// - Extend constants
    /// - Free vars are ignored for now since this is mainly intended
    ///   for extending modules (where there are no free vars) and not
    ///   functions
    ///
    /// IMPORTANT: ALL instructions that hold a const index MUST be
    ///            updated here.
    pub fn extend(&mut self, mut code: Self) {
        use Inst::LoadConst;
        let mut replacements = vec![];
        let const_offset = self.constants.len();
        for (addr, inst) in code.iter_chunk().enumerate() {
            if let LoadConst(index) = inst {
                replacements.push((addr, LoadConst(const_offset + index)));
            }
        }
        for (addr, inst) in replacements {
            code.replace_inst(addr, inst);
        }
        self.chunk.extend(code.chunk);
        self.constants.extend(code.constants);
    }

    /// Get docstring for code unit, if there is one.
    pub fn get_doc(&self) -> ObjectRef {
        if let Some(Inst::LoadConst(0)) = self.chunk.get(1) {
            if let Some(obj_ref) = self.get_const(0) {
                let obj = obj_ref.read().unwrap();
                if let Some(doc) = obj.get_str_val() {
                    return new::str(format_doc(doc));
                }
            }
        }
        new::nil()
    }

    // Instructions ----------------------------------------------------

    pub fn len_chunk(&self) -> usize {
        self.chunk.len()
    }

    pub fn iter_chunk(&self) -> Iter<'_, Inst> {
        self.chunk.iter()
    }

    pub fn push_inst(&mut self, inst: Inst) {
        self.chunk.push(inst)
    }

    pub fn pop_inst(&mut self) -> Option<Inst> {
        self.chunk.pop()
    }

    pub fn insert_inst(&mut self, index: usize, inst: Inst) {
        self.chunk.insert(index, inst);
    }

    pub fn replace_inst(&mut self, index: usize, inst: Inst) {
        self.chunk[index] = inst;
    }

    /// Explicit return statements need to jump to the end of the
    /// function so that the function can be cleanly exited.
    pub fn fix_up_explicit_returns(&mut self) {
        let return_addr = self.len_chunk();
        for addr in 0..return_addr {
            let inst = &self.chunk[addr];
            if let Inst::ReturnPlaceholder(inst_addr, depth) = inst {
                let rel_addr = return_addr - inst_addr;
                self.replace_inst(*inst_addr, Inst::Jump(rel_addr, true, depth - 1));
            }
        }
    }

    // Constants -------------------------------------------------------

    pub fn add_const(&mut self, val_ref: ObjectRef) -> usize {
        let val_guard = val_ref.read().unwrap();
        let val = &*val_guard;

        // XXX: Functions are immutable and comparable, but it feels
        //      potentially unsafe to treat them as such here.
        let is_comparable = val.is_immutable() && !val.is_func();

        for (index, other_ref) in self.iter_constants().enumerate() {
            let other = other_ref.read().unwrap();
            let other_is_comparable = other.is_immutable() && !other.is_func();
            if is_comparable && other_is_comparable && other.is_equal(val) {
                return index;
            }
        }

        let index = self.constants.len();
        drop(val_guard);
        self.constants.push(val_ref);
        index
    }

    pub fn get_const(&self, index: usize) -> Option<&ObjectRef> {
        self.constants.get(index)
    }

    pub fn iter_constants(&self) -> Iter<'_, ObjectRef> {
        self.constants.iter()
    }

    pub fn get_main(&self) -> Option<ObjectRef> {
        let maybe_index = self.constants.iter().position(|obj_ref| {
            let obj = obj_ref.read().unwrap();
            if let Some(func) = obj.down_to_func() {
                func.name() == "$main"
            } else {
                false
            }
        });
        maybe_index.map(|index| self.constants[index].clone())
    }

    // Vars ------------------------------------------------------------

    pub fn free_vars(&self) -> &Vec<FreeVarEntry> {
        &self.free_vars
    }

    /// Add a free var, a reference to a var defined in an enclosing
    /// scope. This also adds a placeholder instruction for the free
    /// var that will replaced in the compiler's name resolution stage.
    pub fn add_free_var<S: Into<String>>(
        &mut self,
        name: S,
        start: Location,
        end: Location,
    ) {
        let addr = self.len_chunk();
        let name = name.into();
        self.free_vars.push((addr, name.clone(), start, end));
        self.push_inst(Inst::FreeVarPlaceholder(addr, name));
    }
}

/// NOTE: When adding or removing instructions, the PartialEq impl
///       below must also be updated.
#[derive(Debug)]
pub enum Inst {
    NoOp,

    // Pop TOS and discard it.
    Pop,

    ScopeStart,
    ScopeEnd,

    StatementStart(Location, Location),

    // Other constants are local to a given code unit.
    LoadConst(usize),

    DeclareVar(String),
    AssignVar(String),

    // Args: name, offset
    //
    // `offset` is the number of scopes above the current scope to start
    // the search. 0 means the current scope, 1 means the parent scope,
    // and so on.
    LoadVar(String, usize),

    // Load module global
    LoadGlobal(String),

    // Load builtin
    LoadBuiltin(String),

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

    // If top of stack is true, jump to address. Otherwise, continue.
    JumpIf(usize, bool, usize),

    // If top of stack is false, jump to address. Otherwise, continue.
    JumpIfNot(usize, bool, usize),

    // If top of stack is NOT nil, jump to address. Otherwise, continue.
    JumpIfNotNil(usize, bool, usize),

    UnaryOp(UnaryOperator),
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

    // Pop TOS and print it to stdout or stderr. Behavior is controlled
    // by passing in flags. Pass `PrintFlags::default()` for the default
    // behavior, which is to print to stdout with no newline.
    Print(PrintFlags),

    DisplayStack(String),
}

bitflags! {
    #[derive(Default)]
    pub struct PrintFlags: u32 {
        const ERR  =   0b00000001; // print to stderr
        const NL   =   0b00000010; // print a trailing newline.
        const REPR =   0b00000100; // print repr using fmt::Debug
        const NO_NIL = 0b00001000; // don't print obj if it's nil
    }
}

impl PartialEq for Inst {
    fn eq(&self, other: &Self) -> bool {
        use Inst::*;

        match (self, other) {
            (NoOp, NoOp) => true,
            (Pop, Pop) => true,
            (ScopeStart, ScopeStart) => true,
            (ScopeEnd, ScopeEnd) => true,
            (StatementStart(..), StatementStart(..)) => true,
            (LoadConst(a), LoadConst(b)) => a == b,
            (DeclareVar(a), DeclareVar(b)) => a == b,
            (AssignVar(a), AssignVar(b)) => a == b,
            (LoadVar(a, i), LoadVar(b, j)) => (a, i) == (b, j),
            (AssignCell(a), AssignCell(b)) => a == b,
            (LoadCell(a), LoadCell(b)) => a == b,
            (LoadCaptured(a), LoadCaptured(b)) => a == b,
            (Jump(a, b, c), Jump(d, e, f)) => (a, b, c) == (d, e, f),
            (JumpPushNil(a, b, c), JumpPushNil(d, e, f)) => (a, b, c) == (d, e, f),
            (JumpIfNot(a, b, c), JumpIfNot(d, e, f)) => (a, b, c) == (d, e, f),
            (UnaryOp(a), UnaryOp(b)) => a == b,
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
            (Print(a), Print(b)) => a == b,
            _ => false,
        }
    }
}
