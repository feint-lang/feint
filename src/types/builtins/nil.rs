//! Built in nil type
use std::fmt;
use std::rc::Rc;

use builtin_object_derive::BuiltinObject;

use super::super::class::Type;
use super::super::object::{Object, ObjectExt};

/// Built in nil type
#[derive(Debug, PartialEq, BuiltinObject)]
pub struct Nil {
    class: Rc<Type>,
}

impl Nil {
    pub fn new(class: Rc<Type>) -> Self {
        Self { class: class.clone() }
    }
}

impl fmt::Display for Nil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "nil")
    }
}
