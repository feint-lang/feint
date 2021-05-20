use std::rc::Rc;

use crate::types::Object;

/// Object store
pub struct ObjectStore {
    storage: Vec<Rc<Object>>,
}

impl ObjectStore {
    pub fn new() -> Self {
        Self { storage: Vec::new() }
    }

    pub fn add(&mut self, object: Rc<Object>) -> usize {
        let index = self.storage.len();
        self.storage.push(object.clone());
        return index;
    }

    pub fn get(&self, index: usize) -> Option<&Rc<Object>> {
        self.storage.get(index)
    }
}
