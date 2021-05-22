use std::collections::HashMap;
use std::rc::Rc;

use crate::types::Object;

/// VM call stack frame.
pub struct Frame {
    parameters: HashMap<String, Rc<dyn Object>>,
    locals: HashMap<String, Rc<dyn Object>>,
    return_address: usize,
}

impl Frame {
    pub fn new(return_address: usize) -> Self {
        Frame { parameters: HashMap::new(), locals: HashMap::new(), return_address }
    }
}
