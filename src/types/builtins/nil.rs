//! Built in nil type
use std::fmt;
use std::rc::Rc;

use builtin_object_derive::BuiltinObject;

use crate::vm::{RuntimeError, RuntimeResult, VM};

use super::super::class::{Type, TypeRef};
use super::super::object::{Object, ObjectExt, ObjectRef};

/// Built in nil type
#[derive(Debug, PartialEq, BuiltinObject)]
pub struct Nil {
    class: TypeRef,
}

impl Nil {
    pub fn new(class: TypeRef) -> Self {
        Self { class: class.clone() }
    }
}

impl fmt::Display for Nil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "nil")
    }
}
