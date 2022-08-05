use std::ops::Index;
use std::slice::Iter;

use crate::types::{create, ObjectRef};

use super::inst::Inst;
use super::result::RuntimeErr;

/// depth, name, value
pub type Locals = Vec<(usize, String, ObjectRef)>;

/// Represents a unit of code.
pub struct Code {
    chunk: Vec<Inst>,
    constants: Vec<ObjectRef>,
}

impl Index<usize> for Code {
    type Output = Inst;

    fn index(&self, index: usize) -> &Self::Output {
        &self.chunk[index]
    }
}

impl Code {
    pub fn new() -> Self {
        Self { chunk: Vec::new(), constants: Vec::new(), locals: Vec::new() }
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

    pub fn replace_inst(&mut self, index: usize, inst: Inst) {
        self.chunk[index] = inst;
    }

    pub fn get_inst(&mut self, index: usize) -> Option<&Inst> {
        self.chunk.get(index)
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
            Err(RuntimeErr::new_object_not_found_err(index))
        }
    }

    pub fn iter_constants(&self) -> Iter<'_, ObjectRef> {
        self.constants.iter()
    }

    // Locals ----------------------------------------------------------

    pub fn clone_locals(&self) -> Vec<(usize, String, ObjectRef)> {
        self.locals.clone()
    }

    /// Add local at depth with initial value of nil.
    pub fn add_local(&mut self, depth: usize, name: &str) {
        if self.find_local_at_depth(depth, name).is_none() {
            self.locals.push((depth, name.to_string(), create::new_nil()))
        }
    }

    pub fn find_local(&self, name: &str) -> Option<ObjectRef> {
        self.locals
            .iter()
            .rev()
            .position(|(_, n, _)| name == n)
            .map(|i| self.locals[i].2.clone())
    }

    pub fn find_local_at_depth(&self, depth: usize, name: &str) -> Option<ObjectRef> {
        self.locals
            .iter()
            .rev()
            .position(|(d, n, _)| &depth == d && name == n)
            .map(|i| self.locals[i].2.clone())
    }
}
