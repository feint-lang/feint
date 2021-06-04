//! Built in nil type
use std::any::Any;
use std::fmt;

use crate::vm::{RuntimeBoolResult, RuntimeContext};

use super::class::TypeRef;
use super::object::{Object, ObjectExt};

pub struct Nil {
    class: TypeRef,
}

impl Nil {
    pub fn new(class: TypeRef) -> Self {
        Self { class }
    }
}

impl Object for Nil {
    fn class(&self) -> &TypeRef {
        &self.class
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    // Unary operations -----------------------------------------------

    fn as_bool(&self, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        Ok(false)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Nil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "nil")
    }
}

impl fmt::Debug for Nil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
