use std::collections::HashMap;
use std::rc::Rc;

use crate::types::ObjectRef;

/// Object store
pub struct ObjectStore {
    storage: Vec<ObjectRef>,
    names: Vec<String>,
    name_map: HashMap<String, usize>,
}

impl ObjectStore {
    pub fn new() -> Self {
        Self { storage: Vec::new(), names: Vec::new(), name_map: HashMap::new() }
    }

    pub fn add(&mut self, object: ObjectRef) -> usize {
        let index = self.storage.len();
        self.storage.push(object.clone());
        index
    }

    pub fn get(&self, index: usize) -> Option<&ObjectRef> {
        self.storage.get(index)
    }

    /// Add a name. This is used to store names in the compiler so that
    /// they can be retrieved by the VM when it executes an assignment
    /// instruction.
    ///
    /// FIXME: This scheme currently only allows a flat namespace.
    pub fn add_name<S: Into<String>>(&mut self, name: S) -> usize {
        let name = name.into();
        let current = self.names.iter().position(|n| *n == name);
        if let Some(index) = current {
            return index;
        }
        let index = self.names.len();
        self.names.push(name);
        index
    }

    /// Get a name from the list of names added via add_name().
    pub fn get_name(&self, index: usize) -> &String {
        self.names.get(index).unwrap()
    }

    /// This adds a pointer from a name to an object.
    pub fn set_index_for_name(&mut self, name_index: usize, index: usize) {
        let name = self.get_name(name_index);
        self.name_map.insert(name.clone(), index);
    }

    /// This gets the pointer for a name so the referenced object can be
    /// retrieved.
    pub fn get_index_for_name(&mut self, name: &str) -> Option<&usize> {
        self.name_map.get(name)
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
