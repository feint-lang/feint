//! Built in function type
use std::any::Any;
use std::fmt;

use crate::vm::Chunk;

use super::class::TypeRef;
use super::object::{Object, ObjectExt, ObjectRef};

pub struct Func {
    class: TypeRef,
    name: String,
    params: Vec<String>,
    pub chunk: Chunk,
}

impl Func {
    pub fn new<S: Into<String>>(
        class: TypeRef,
        name: S,
        params: Vec<String>,
        chunk: Chunk,
    ) -> Self {
        Self { class, name: name.into(), params, chunk }
    }
}

impl Object for Func {
    fn class(&self) -> &TypeRef {
        &self.class
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}) -> ...", self.name, self.params.len())
    }
}

impl fmt::Debug for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
