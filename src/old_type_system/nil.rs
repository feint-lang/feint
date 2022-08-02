//! Nil type
use std::any::Any;
use std::fmt;

use crate::vm::{RuntimeBoolResult, RuntimeContext};

use super::builtin_types::BUILTIN_TYPES;
use super::class::TypeRef;
use super::object::Object;

pub struct Nil {}

impl Nil {
    pub fn new() -> Self {
        Self {}
    }
}

impl Object for Nil {
    fn class(&self) -> &TypeRef {
        BUILTIN_TYPES.get("Nil").unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    // Unary operations -----------------------------------------------

    fn bool_val(&self, _ctx: &RuntimeContext) -> RuntimeBoolResult {
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
        write!(f, "{self}")
    }
}
