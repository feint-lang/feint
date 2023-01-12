use std::ops::Index;
use std::slice::Iter;

use crate::types::{new, FuncTrait, ObjectRef};
use crate::util::{format_doc, Location};

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
    ///
    /// IMPORTANT: ALL instructions that hold a const index MUST be
    ///            updated here.
    pub fn extend(&mut self, mut code: Self) {
        use Inst::{LoadConst, MakeFunc};
        let mut replacements = vec![];
        let const_offset = self.constants.len();
        for (addr, inst) in code.iter_chunk().enumerate() {
            if let LoadConst(index) = inst {
                replacements.push((addr, LoadConst(const_offset + index)));
            } else if let MakeFunc(index) = inst {
                replacements.push((addr, MakeFunc(const_offset + index)));
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
            if let Ok(obj_ref) = self.get_const(0) {
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
                log::trace!("Constant {other} found @ index {index}; not adding");
                return index;
            }
        }

        let index = self.constants.len();
        log::trace!("Constant {val} not found; adding @ index {index}");
        drop(val_guard);
        self.constants.push(val_ref);
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
