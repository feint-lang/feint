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
}
