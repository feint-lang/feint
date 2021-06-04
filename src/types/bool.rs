//! Built in boolean type
use std::any::Any;
use std::fmt;

use crate::vm::{RuntimeBoolResult, RuntimeContext, RuntimeErr};

use super::class::TypeRef;
use super::object::{Object, ObjectExt, ObjectRef};

pub struct Bool {
    class: TypeRef,
    value: bool,
}

impl Bool {
    pub fn new(class: TypeRef, value: bool) -> Self {
        Self { class, value }
    }

    pub fn value(&self) -> &bool {
        &self.value
    }
}

impl Object for Bool {
    fn class(&self) -> &TypeRef {
        &self.class
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    // Unary operations -----------------------------------------------

    fn as_bool(&self, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        Ok(*self.value())
    }

    // Binary operations -----------------------------------------------

    fn is_equal(&self, rhs: ObjectRef, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            Ok(self.is(rhs) || self.value() == rhs.value())
        } else {
            Err(RuntimeErr::new_type_error(format!(
                "Could not compare Bool to {} for equality",
                rhs.class().name()
            )))
        }
    }

    fn and(&self, rhs: ObjectRef, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            Ok(*self.value() && *rhs.value())
        } else {
            Err(RuntimeErr::new_type_error(format!(
                "Bool && {} not implemented",
                rhs.class().name()
            )))
        }
    }

    fn or(&self, rhs: ObjectRef, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            Ok(*self.value() || *rhs.value())
        } else {
            Err(RuntimeErr::new_type_error(format!(
                "Bool || {} not implemented",
                rhs.class().name()
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
