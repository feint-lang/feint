use std::ops::Index;
use std::slice::Iter;

use crate::types::ObjectRef;
use crate::util::Location;

use super::inst::Inst;
use super::result::RuntimeErr;

type CellVarEntry = Vec<(
    usize,  // address
    String, // name
    usize,  // local index
)>;

type FreeVarEntry = Vec<(
    usize,    // address
    String,   // name
    Location, // source start
    Location, // source end
)>;

/// Code for a module or function.
pub struct Code {
    chunk: Vec<Inst>,
    constants: Vec<ObjectRef>,
    // Vars defined in this unit of code that are referenced by free
    // vars in other units of code.
    cell_vars: CellVarEntry,
    // Vars defined outside of this unit of code.
    free_vars: FreeVarEntry,
}

impl Index<usize> for Code {
    type Output = Inst;

    fn index(&self, index: usize) -> &Self::Output {
        &self.chunk[index]
    }
}

impl Code {
    pub fn new() -> Self {
        Self { chunk: vec![], constants: vec![], cell_vars: vec![], free_vars: vec![] }
    }

    /// Initialize code object with a list of instructions, also known
    /// as a chunk.
    pub fn with_chunk(chunk: Vec<Inst>) -> Self {
        let mut code = Code::new();
        code.chunk.extend(chunk);
        code
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

    pub fn insert_inst(&mut self, index: usize, inst: Inst) {
        self.chunk.insert(index, inst);
    }

    /// Insert instruction at `len - 1`, pushing the last instruction
    /// to the end.
    pub fn insert_inst_last(&mut self, inst: Inst) {
        let len = self.len_chunk();
        if len == 0 {
            self.push_inst(inst);
        } else {
            self.chunk.insert(len - 1, inst);
        }
    }

    pub fn replace_inst(&mut self, index: usize, inst: Inst) {
        self.chunk[index] = inst;
    }

    pub fn get_inst(&mut self, index: usize) -> Option<&Inst> {
        self.chunk.get(index)
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

    pub fn add_const(&mut self, constant: ObjectRef) -> usize {
        let index = self.constants.len();
        self.constants.push(constant);
        index
    }

    pub fn get_const(&self, index: usize) -> Result<&ObjectRef, RuntimeErr> {
        if let Some(obj) = self.constants.get(index) {
            Ok(obj)
        } else {
            Err(RuntimeErr::constant_not_found(index))
        }
    }

    pub fn iter_constants(&self) -> Iter<'_, ObjectRef> {
        self.constants.iter()
    }

    // Vars ------------------------------------------------------------

    pub fn cell_vars(&self) -> &CellVarEntry {
        &self.cell_vars
    }

    pub fn add_cell_var<S: Into<String>>(&mut self, name: S, index: usize) {
        let addr = self.len_chunk();
        self.cell_vars.push((addr, name.into(), index));
    }

    pub fn free_vars(&self) -> &FreeVarEntry {
        &self.free_vars
    }

    /// Add a free var, a reference to a var defined in an enclosing
    /// scope. This also pushes a placeholder instruction for the var.
    pub fn add_free_var<S: Into<String>>(
        &mut self,
        name: S,
        start: Location,
        end: Location,
    ) {
        let addr = self.len_chunk();
        let name = name.into();
        self.free_vars.push((addr, name.clone(), start, end));
        self.push_inst(Inst::VarPlaceholder(addr, name));
    }

    pub fn has_free_var_at_addr(&self, addr: usize) -> bool {
        self.free_vars().iter().any(|(a, ..)| *a == addr)
    }
}
