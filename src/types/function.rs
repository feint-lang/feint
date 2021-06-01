//! Built in function type
use std::any::Any;
use std::fmt;

use crate::vm::{
    Instructions, RuntimeBoolResult, RuntimeContext, RuntimeError, RuntimeResult,
};

use super::class::TypeRef;
use super::object::{Object, ObjectExt, ObjectRef};

#[derive(Debug)]
pub struct Function {
    class: TypeRef,
    name: String,
    parameters: Vec<String>,
    instructions: Instructions,
}

impl Function {
    pub fn new<S: Into<String>>(
        class: TypeRef,
        name: S,
        parameters: Vec<String>,
        instructions: Instructions,
    ) -> Self {
        Self { class, name: name.into(), parameters, instructions }
    }

    pub fn call(&self, this: Option<ObjectRef>, args: Vec<ObjectRef>) {}
}

impl Object for Function {
    fn class(&self) -> &TypeRef {
        &self.class
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}()", self.name)
    }
}
