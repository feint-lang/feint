use std::collections::HashMap;

use crate::types::ObjectRef;
use crate::vm::objects::Objects;
use crate::vm::RuntimeErr;

pub struct Namespace {
    name_index: HashMap<String, usize>, // name => object index
    objects: Objects,
}

impl Namespace {
    pub fn new() -> Self {
        Namespace { name_index: HashMap::new(), objects: Objects::default() }
    }

    pub fn clear(&mut self) {
        self.name_index.clear();
        self.objects.clear();
    }

    // Objects ---------------------------------------------------------

    pub fn size(&self) -> usize {
        self.objects.size()
    }

    pub fn add_obj(&mut self, obj: ObjectRef) -> usize {
        self.objects.add(obj)
    }

    pub fn set_obj(
        &mut self,
        index: usize,
        obj: ObjectRef,
    ) -> Result<usize, RuntimeErr> {
        self.objects.replace(index, obj)
    }

    pub fn get_obj(&self, index: usize) -> Result<&ObjectRef, RuntimeErr> {
        self.objects.get(index)
    }

    // Vars ------------------------------------------------------------
    //
    // Vars are named "pointers" to objects.

    /// Add a var, settings its initial value to nil.
    pub fn add_var<S: Into<String>>(&mut self, name: S) -> Result<usize, RuntimeErr> {
        let nil = self.get_obj(0)?.clone();
        let index = self.size();
        self.name_index.insert(name.into(), index);
        self.add_obj(nil);
        Ok(index)
    }

    /// Set a var's value.
    pub fn set_var(&mut self, name: &str, obj: ObjectRef) -> Result<usize, RuntimeErr> {
        let index = self.var_index(name)?;
        self.set_obj(index, obj)?;
        Ok(index)
    }

    /// Get the object index for a var.
    pub fn var_index(&self, name: &str) -> Result<usize, RuntimeErr> {
        if let Some(index) = self.name_index.get(name) {
            Ok(*index)
        } else {
            let message = format!("Name not defined in current scope: {name}");
            Err(RuntimeErr::new_name_err(message))
        }
    }
}
