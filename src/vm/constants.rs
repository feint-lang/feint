use std::slice::Iter;

use crate::types::ObjectRef;

use super::result::RuntimeErr;

pub struct Constants {
    storage: Vec<ObjectRef>,
}

impl Constants {
    pub fn new() -> Self {
        Self { storage: Vec::new() }
    }

    pub fn iter(&self) -> Iter<'_, ObjectRef> {
        self.storage.iter()
    }

    pub fn add(&mut self, object: ObjectRef) -> usize {
        let index = self.storage.len();
        self.storage.push(object.clone());
        index
    }

    pub fn get(&self, index: usize) -> Result<&ObjectRef, RuntimeErr> {
        if let Some(obj) = self.storage.get(index) {
            Ok(obj)
        } else {
            Err(RuntimeErr::constant_not_found(index))
        }
    }
}
