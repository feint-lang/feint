use std::collections::HashMap;
use std::rc::Rc;

use crate::types::ComplexObject;

/// VM call stack frame.
pub struct Frame {
    parameters: HashMap<String, Rc<ComplexObject>>,
    locals: HashMap<String, Rc<ComplexObject>>,
    return_address: usize,
}

impl Frame {
    pub fn new(return_address: usize) -> Self {
        Frame { parameters: HashMap::new(), locals: HashMap::new(), return_address }
    }
}
