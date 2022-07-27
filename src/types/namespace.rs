use std::any::Any;
use std::collections::HashMap;

use super::class::TypeRef;
use super::object::Object;
use super::{ObjectRef, BUILTIN_TYPES};

// Namespace -----------------------------------------------------------

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

    pub fn set_obj(&mut self, index: usize, obj: ObjectRef) -> Option<usize> {
        self.objects.replace(index, obj)
    }

    pub fn get_obj(&self, index: usize) -> Option<&ObjectRef> {
        self.objects.get(index)
    }

    // Vars ------------------------------------------------------------
    //
    // Vars are named "pointers" to objects.

    /// Add a var, settings its initial value to nil.
    pub fn add_var<S: Into<String>>(&mut self, name: S) -> Option<usize> {
        let nil = self.get_obj(0).unwrap().clone();
        let index = self.size();
        self.name_index.insert(name.into(), index);
        self.add_obj(nil);
        Some(index)
    }

    /// Set a var's value.
    pub fn set_var(&mut self, name: &str, obj: ObjectRef) -> Option<usize> {
        if let Some(index) = self.var_index(name) {
            if let Some(index) = self.set_obj(index, obj) {
                Some(index)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get a var.
    pub fn get_var(&mut self, name: &str) -> Option<&ObjectRef> {
        if let Some(index) = self.var_index(name) {
            self.get_obj(index)
        } else {
            None
        }
    }

    /// Get the object index for a var.
    pub fn var_index(&self, name: &str) -> Option<usize> {
        if let Some(index) = self.name_index.get(name) {
            Some(*index)
        } else {
            None
        }
    }
}

impl Object for Namespace {
    fn class(&self) -> &TypeRef {
        BUILTIN_TYPES.get("Namespace").unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Object --------------------------------------------------------------

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

    pub fn size(&self) -> usize {
        self.storage.len()
    }

    pub fn add(&mut self, object: ObjectRef) -> usize {
        let index = self.storage.len();
        self.storage.push(object.clone());
        index
    }

    pub fn replace(&mut self, index: usize, obj: ObjectRef) -> Option<usize> {
        if let Some(_) = self.storage.get(index) {
            self.storage[index] = obj;
            Some(index)
        } else {
            None
        }
    }

    pub fn get(&self, index: usize) -> Option<&ObjectRef> {
        if let Some(obj) = self.storage.get(index) {
            Some(obj)
        } else {
            None
        }
    }
}

impl Default for Objects {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
