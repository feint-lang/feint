use std::fmt;
use std::rc::Rc;

use builtin_object_derive::BuiltinObject;

use crate::vm::{RuntimeError, RuntimeResult};

use super::super::class::{Type, TypeRef};
use super::super::object::{Object, ObjectExt, ObjectRef};

/// Built in boolean type
#[derive(Debug, PartialEq, BuiltinObject)]
pub struct Bool {
    class: TypeRef,
    value: bool,
}

impl Bool {
    pub fn new(class: TypeRef, value: bool) -> Self {
        Self { class: class.clone(), value }
    }

    pub fn value(&self) -> &bool {
        &self.value
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
