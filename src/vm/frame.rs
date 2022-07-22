use std::collections::HashMap;

use crate::types::{Func, ObjectRef};
use crate::vm::Chunk;

/// VM call stack frame.
pub struct Frame {
    func: ObjectRef,
    args: Vec<ObjectRef>,
    return_addr: usize,
}

impl Frame {
    pub fn new(func: ObjectRef, args: Vec<ObjectRef>, return_addr: usize) -> Self {
        Frame { func, args, return_addr }
    }

    pub fn chunk(&self) -> Option<Chunk> {
        if let Some(func) = self.func.as_any().downcast_ref::<Func>() {
            // return Some(func.chunk);
        }
        None
    }
}
