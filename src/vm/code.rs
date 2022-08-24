use std::ops::Index;
use std::slice::Iter;

use crate::types::{FuncTrait, ObjectRef};
use crate::util::Location;

use super::inst::Inst;
use super::result::RuntimeErr;

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

impl Index<usize> for Code {
    type Output = Inst;

    fn index(&self, index: usize) -> &Self::Output {
        &self.chunk[index]
    }
}

impl Code {
    pub fn new() -> Self {
        Self { chunk: vec![], constants: vec![], free_vars: vec![] }
    }

    /// Initialize code object with a list of instructions, also known
    /// as a chunk.
    pub fn with_chunk(chunk: Vec<Inst>) -> Self {
        let mut code = Code::new();
        code.chunk.extend(chunk);
        code
    }

    /// Extend this `Code` object with another `Code` object:
    ///
    /// - Extend instructions, adjusting constant indexes
    /// - Extend constants
    /// - Free vars are ignored for now since this is mainly intended
    ///   for extending modules (where there are no free vars) and not
    ///   functions
    pub fn extend(&mut self, mut code: Self) {
        let mut replacements = vec![];
        let const_offset = self.constants.len();
        for (addr, inst) in code.iter_chunk().enumerate() {
            if let Inst::LoadConst(index) = inst {
                replacements.push((addr, *index));
            }
        }
        for (addr, index) in replacements {
            code.replace_inst(addr, Inst::LoadConst(index + const_offset));
        }
        self.chunk.extend(code.chunk);
        self.constants.extend(code.constants);
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

    pub fn has_main(&self) -> bool {
        self.constants.iter().any(|obj_ref| {
            let obj = obj_ref.read().unwrap();
            if let Some(func) = obj.down_to_func() {
                func.name() == "$main"
            } else {
                false
            }
        })
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
