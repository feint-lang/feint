use std::rc::Rc;

use crate::types::Object;

/// Object store
pub struct ObjectStore {
    storage: Vec<Rc<dyn Object>>,
}

impl ObjectStore {
    pub fn new() -> Self {
        Self { storage: Vec::new() }
    }

    pub fn add(&mut self, object: Rc<dyn Object>) -> usize {
        let index = self.storage.len();
        self.storage.push(object.clone());
        return index;
    }

    pub fn get(&self, index: usize) -> Option<&Rc<dyn Object>> {
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

        let int = Rc::new(builtins.new_int(0));
        let int_copy = int.clone();

        let index = store.add(int);

        let retrieved = store.get(index).unwrap();
        let retrieved = retrieved.as_any().downcast_ref::<Int>().unwrap();

        assert_eq!(retrieved.class().id(), int_copy.class().id());
        assert_eq!(retrieved.id(), int_copy.id());
        assert_eq!(retrieved, int_copy.as_ref());
    }
}
