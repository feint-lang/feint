use crate::types::{Args, ObjectRef};

/// VM call stack frame.
pub struct Frame {
    func: ObjectRef,
    args: Args,
    return_addr: usize,
}

impl Frame {
    pub fn new(func: ObjectRef, args: Args, return_addr: usize) -> Self {
        Frame { func, args, return_addr }
    }
}
