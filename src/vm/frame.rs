use std::collections::HashMap;

use crate::types::ObjectRef;

/// VM call stack frame.
pub struct Frame {
    parameters: HashMap<String, ObjectRef>,
    locals: HashMap<String, ObjectRef>,
    return_address: usize,
}

impl Frame {
    pub fn new(return_address: usize) -> Self {
        Frame { parameters: HashMap::new(), locals: HashMap::new(), return_address }
    }
}
