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
        if let Some(_) = self.storage.get(index) {
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

#[cfg(test)]
mod tests {
    use crate::types::{Object, ObjectExt};
    use crate::vm::context::RuntimeContext;

    #[test]
    fn test_add_retrieve() {
        let mut ctx = RuntimeContext::default();
        let int = ctx.builtins.new_int(0);
        let int_copy = int.clone();
        let index = ctx.add_const(int.clone());
        let retrieved = ctx.get_const(index).unwrap();

        let retrieved = retrieved.lock().unwrap();
        // let int_copy

        // assert!(retrieved.is_equal(&int_copy, &ctx).unwrap());

        // let int_copy = int_copy.lock().unwrap();
        // let int_class = int_copy.class().lock().unwrap();
        // let retrieved_class = retrieved.class().lock().unwrap();

        // assert!(retrieved_class.is(&int_class));
        // assert_eq!(retrieved_class.id(), int_class.id());

        // assert!(retrieved
        //     .class()
        //     .lock()
        //     .unwrap()
        //     .is_equal(&int_copy.lock().unwrap().class(), &ctx)
        //     .unwrap());
        //
        // assert!(retrieved_class.is_equal(&int_copy_class, &ctx).unwrap());
        //
        // assert!(retrieved.is(&int_copy.lock().unwrap()));
        // assert_eq!(retrieved.id(), int_copy.lock().unwrap().id());
        // assert!(retrieved.is_equal(&int_copy, &ctx).unwrap());
    }
}
