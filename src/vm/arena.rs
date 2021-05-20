use std::rc::Rc;

use crate::types::Object;

/// Object store
pub struct ObjectStore<'a> {
    storage: Vec<Rc<Object<'a>>>,
}

impl<'a> ObjectStore<'a> {
    pub fn new() -> Self {
        Self { storage: Vec::new() }
    }

    pub fn add(&mut self, object: Rc<Object<'a>>) -> usize {
        let index = self.storage.len();
        self.storage.push(object);
        return index;
    }

    pub fn get(&self, index: usize) -> Option<&Rc<Object>> {
        self.storage.get(index)
    }
}
