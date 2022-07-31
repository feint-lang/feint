use std::any::Any;
use std::collections::HashMap;
use std::fmt;

use crate::vm::RuntimeContext;

use super::builtin_types::BUILTIN_TYPES;
use super::class::TypeRef;
use super::object::{Object, ObjectExt, ObjectRef};
use super::result::GetAttrResult;

// Namespace -----------------------------------------------------------

pub struct Namespace {
    objects: HashMap<String, ObjectRef>,
}

impl Namespace {
    pub fn new() -> Self {
        Namespace { objects: HashMap::new() }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn size(&self) -> usize {
        self.objects.len()
    }

    /// Add a var, settings its initial value as specified (usually
    /// nil).
    pub fn add_var<S: Into<String>>(&mut self, name: S, initial: ObjectRef) {
        self.objects.insert(name.into(), initial);
    }

    /// Set a var's value. This will only succeed if the var already
    /// exists.
    pub fn set_var(&mut self, name: &str, obj: ObjectRef) -> bool {
        if self.objects.contains_key(name) {
            self.objects.insert(name.to_owned(), obj);
            true
        } else {
            false
        }
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
            Err(self.attr_does_not_exist(name))
        }
    }

    fn is_equal(&self, rhs: &dyn Object, ctx: &RuntimeContext) -> bool {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            if self.is(&rhs) {
                true
            } else {
                let lhs_objects = &self.objects;
                let rhs_objects = &rhs.objects;
                if lhs_objects.len() != rhs_objects.len() {
                    // Namespaces have a different number of entries, so
                    // they can't be equal.
                    false
                } else if !lhs_objects.keys().all(|k| rhs_objects.contains_key(k)) {
                    // Namespaces have differing keys, so they can't be
                    // equal.
                    false
                } else {
                    // Otherwise, compare all entries for equality.
                    for (name, lhs_val) in lhs_objects.iter() {
                        let rhs_val = &rhs_objects[name];
                        if !lhs_val.is_equal(&**rhs_val, ctx) {
                            return false;
                        }
                    }
                    true
                }
            }
        } else {
            false
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = self.class().qualified_name();
        let id = self.id();
        write!(f, "<{type_name}> @ {id}")
    }
}

impl fmt::Debug for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
