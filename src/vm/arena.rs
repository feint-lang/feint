use std::rc::Rc;

use crate::types::ObjectRef;

/// Object store
pub struct ObjectStore {
    storage: Vec<ObjectRef>,
}

impl ObjectStore {
    pub fn new() -> Self {
        Self { storage: Vec::new() }
    }

    pub fn add(&mut self, object: ObjectRef) -> usize {
        let index = self.storage.len();
        self.storage.push(object.clone());
        return index;
    }

    pub fn get(&self, index: usize) -> Option<&ObjectRef> {
        self.storage.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::builtins::Int;
    use crate::types::{Builtins, Object};

    #[test]
    fn test_add_retrieve() {
        let builtins = Builtins::new();
        let mut store = ObjectStore::new();
        let int = builtins.new_int(0);
        let int_copy = int.clone();
        let index = store.add(int);
        let retrieved = store.get(index).unwrap();
        assert_eq!(retrieved.class().id(), int_copy.class().id());
        assert_eq!(retrieved.id(), int_copy.id());
        // assert!(retrieved.is_equal(int_copy).unwrap());
    }
}
