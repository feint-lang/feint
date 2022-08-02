use std::slice::Iter;

use crate::types::ObjectRef;

use super::result::RuntimeErr;

pub struct Objects {
    storage: Vec<ObjectRef>,
}

impl Objects {
    pub fn new(storage: Vec<ObjectRef>) -> Self {
        Self { storage }
    }

    pub fn clear(&mut self) {
        self.storage.clear();
    }

    pub fn iter(&self) -> Iter<'_, ObjectRef> {
        self.storage.iter()
    }

    pub fn size(&self) -> usize {
        self.storage.len()
    }

    pub fn add(&mut self, object: ObjectRef) -> usize {
        let index = self.storage.len();
        self.storage.push(object.clone());
        index
    }

    pub fn replace(
        &mut self,
        index: usize,
        obj: ObjectRef,
    ) -> Result<usize, RuntimeErr> {
        if self.storage.get(index).is_some() {
            self.storage[index] = obj;
            Ok(index)
        } else {
            Err(RuntimeErr::new_object_not_found_err(index))
        }
    }

    pub fn get(&self, index: usize) -> Result<&ObjectRef, RuntimeErr> {
        if let Some(obj) = self.storage.get(index) {
            Ok(obj)
        } else {
            Err(RuntimeErr::new_object_not_found_err(index))
        }
    }
}

impl Default for Objects {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
