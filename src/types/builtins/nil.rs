//! Built in nil type
use std::any::Any;
use std::fmt;

use crate::vm::{RuntimeBoolResult, RuntimeContext, RuntimeResult};

use super::super::class::TypeRef;
use super::super::object::Object;

/// Built in nil type
#[derive(Debug, PartialEq)]
pub struct Nil {
    class: TypeRef,
}

impl Nil {
    pub fn new(class: TypeRef) -> Self {
        Self { class: class.clone() }
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

    fn not(&self, ctx: &RuntimeContext) -> RuntimeResult {
        Ok(ctx.builtins.true_obj.clone())
    }

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
