use std::any::Any;
use std::collections::HashMap;
use std::fmt;

use crate::vm::{RuntimeContext, RuntimeErr};

use super::builtin_types::BUILTIN_TYPES;
use super::class::TypeRef;
use super::object::{Object, ObjectRef};
use super::result::GetAttrResult;

// Namespace -----------------------------------------------------------

pub struct Namespace {
    name: String,
    objects: HashMap<String, ObjectRef>,
    nil_obj: ObjectRef,
}

impl Namespace {
    pub fn new<S: Into<String>>(name: S, nil: ObjectRef) -> Self {
        Namespace { name: name.into(), objects: HashMap::new(), nil_obj: nil }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn size(&self) -> usize {
        self.objects.len()
    }

    /// Add a var, settings its initial value to nil.
    pub fn add_var<S: Into<String>>(&mut self, name: S) {
        self.objects.insert(name.into(), self.nil_obj.clone());
    }

    /// Set a var's value.
    pub fn set_var(&mut self, name: &str, obj: ObjectRef) -> bool {
        if self.objects.contains_key(name) {
            self.objects.insert(name.to_owned(), obj);
            true
        } else {
            false
        }
    }

    /// Add and set a var in one step.
    pub fn add_and_set_var(&mut self, name: &str, obj: ObjectRef) -> bool {
        self.add_var(name);
        self.set_var(name, obj)
    }

    /// Get a var.
    pub fn get_var(&self, name: &str) -> Option<&ObjectRef> {
        self.objects.get(name)
    }
}

impl Object for Namespace {
    fn class(&self) -> &TypeRef {
        BUILTIN_TYPES.get("Namespace").unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_attr(&self, name: &str, ctx: &RuntimeContext) -> GetAttrResult {
        if let Some(attr) = self.get_base_attr(name, ctx) {
            return Ok(attr);
        }
        if let Some(obj) = self.get_var(name) {
            Ok(obj.clone())
        } else {
            Err(RuntimeErr::new_attr_does_not_exist(
                self.qualified_type_name().as_str(),
                name,
            ))
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.as_str();
        let size = self.size();
        let noun = if size == 1 { "entry" } else { "entries" };
        write!(f, "Namespace: {} ({} {noun})", name, size)
    }
}

impl fmt::Debug for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
