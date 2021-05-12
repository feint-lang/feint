use std::collections::HashMap;

use crate::builtins::Object;

/// VM call stack frame.
pub struct Frame<'a> {
    parameters: HashMap<&'a str, &'a Object<'a>>,
    locals: HashMap<&'a str, &'a Object<'a>>,
    return_address: usize,
}

impl<'a> Frame<'a> {
    pub fn new(return_address: usize) -> Self {
        Frame { parameters: HashMap::new(), locals: HashMap::new(), return_address }
    }
}
