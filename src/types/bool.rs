//! Boolean type
use std::any::Any;
use std::fmt;

use crate::vm::{RuntimeBoolResult, RuntimeContext, RuntimeErr};

use super::builtin_types::BUILTIN_TYPES;
use super::class::TypeRef;
use super::object::{Object, ObjectExt};

pub struct Bool {
    value: bool,
}

impl Bool {
    pub fn new(value: bool) -> Self {
        Self { value }
    }

    pub fn value(&self) -> &bool {
        &self.value
    }
}

impl Object for Bool {
    fn class(&self) -> &TypeRef {
        BUILTIN_TYPES.get("Bool").unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    // Unary operations -----------------------------------------------

    fn bool_val(&self, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        Ok(*self.value())
    }

    // Binary operations -----------------------------------------------

    fn is_equal(&self, rhs: &dyn Object, _ctx: &RuntimeContext) -> bool {
        if let Some(rhs) = rhs.down_to_bool() {
            self.is(rhs) || self.value() == rhs.value()
        } else {
            false
        }
    }

    fn and(&self, rhs: &dyn Object, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_bool() {
            Ok(*self.value() && *rhs.value())
        } else {
            Err(RuntimeErr::new_type_err(format!(
                "{} && {} not implemented",
                self.class(),
                rhs.class()
            )))
        }
    }

    fn or(&self, rhs: &dyn Object, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_bool() {
            Ok(*self.value() || *rhs.value())
        } else {
            Err(RuntimeErr::new_type_err(format!(
                "{} || {} not implemented",
                self.class(),
                rhs.class()
            )))
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Debug for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
