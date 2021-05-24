use std::fmt;
use std::rc::Rc;

use builtin_object_derive::BuiltinObject;

use super::super::class::Type;
use super::super::object::{Object, ObjectExt};

/// Built in boolean type
#[derive(Debug, PartialEq, BuiltinObject)]
pub struct Bool {
    class: Rc<Type>,
    value: bool,
}

impl Bool {
    pub fn new(class: Rc<Type>, value: bool) -> Self {
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
