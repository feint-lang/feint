use crate::types::ObjectRef;

pub struct Constants {
    storage: Vec<ObjectRef>,
}

impl Constants {
    pub fn new(storage: Vec<ObjectRef>) -> Self {
        Self { storage }
    }

    pub fn add(&mut self, object: ObjectRef) -> usize {
        let index = self.storage.len();
        self.storage.push(object.clone());
        index
    }

    pub fn replace(&mut self, index: usize, obj: ObjectRef) {
        self.storage[index] = obj;
    }

    pub fn get(&self, index: usize) -> Option<&ObjectRef> {
        self.storage.get(index)
    }
}

impl Default for Constants {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::context::RuntimeContext;

    #[test]
    fn test_add_retrieve() {
        let mut ctx = RuntimeContext::default();
        let int = ctx.builtins.new_int(0);
        let int_copy = int.clone();
        let index = ctx.add_obj(int);
        let retrieved = ctx.get_obj(index).unwrap();
        assert_eq!(retrieved.class().id(), int_copy.class().id());
        assert_eq!(retrieved.id(), int_copy.id());
        assert!(retrieved.is_equal(int_copy, &ctx).unwrap());
    }
}
