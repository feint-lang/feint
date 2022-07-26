//! Built in function type
use std::any::Any;
use std::fmt;

use crate::vm::Chunk;

use super::class::Type;
use super::object::{Object, ObjectRef};
use super::types::TYPES;

pub struct Func {
    name: String,
    params: Vec<String>,
    pub chunk: Chunk,
}

impl Func {
    pub fn new<S: Into<String>>(name: S, params: Vec<String>, chunk: Chunk) -> Self {
        Self { name: name.into(), params, chunk }
    }
}

impl Object for Func {
    fn class(&self) -> &Type {
        TYPES.get("Func").unwrap()
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
