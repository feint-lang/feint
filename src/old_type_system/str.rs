//! String type
use std::any::Any;
use std::fmt;

use crate::vm::{RuntimeBoolResult, RuntimeContext, RuntimeErr, RuntimeObjResult};

use super::builtin_types::BUILTIN_TYPES;
use super::class::TypeRef;
use super::object::{Object, ObjectExt};

pub struct Str {
    value: String,
}

impl Str {
    pub fn new<S: Into<String>>(value: S) -> Self {
        Self { value: value.into() }
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

impl Object for Str {
    fn class(&self) -> &TypeRef {
        BUILTIN_TYPES.get("Str").unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_equal(&self, rhs: &dyn Object, _ctx: &RuntimeContext) -> bool {
        if let Some(rhs) = rhs.down_to_string() {
            self.is(rhs) || self.value() == rhs.value()
        } else {
            false
        }
    }

    fn add(&self, rhs: &dyn Object, ctx: &RuntimeContext) -> RuntimeObjResult {
        if let Some(rhs) = rhs.down_to_string() {
            let a = self.value();
            let b = rhs.value();
            let mut value = String::with_capacity(a.len() + b.len());
            value.push_str(a);
            value.push_str(b);
            let value = ctx.builtins.new_str(value);
            Ok(value)
        } else {
            Err(RuntimeErr::new_type_err(format!(
                "Cannot concatenate {} to {}",
                rhs.class(),
                self.class(),
            )))
        }
    }

    fn less_than(&self, rhs: &dyn Object, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_string() {
            Ok(self.value() < rhs.value())
        } else {
            Err(RuntimeErr::new_type_err(format!(
                "Cannot compare {} to {}: <",
                rhs.class(),
                self.class(),
            )))
        }
    }

    fn greater_than(
        &self,
        rhs: &dyn Object,
        _ctx: &RuntimeContext,
    ) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_string() {
            Ok(self.value() > rhs.value())
        } else {
            Err(RuntimeErr::new_type_err(format!(
                "Cannot compare {} to {}: >",
                rhs.class(),
                self.class(),
            )))
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl fmt::Debug for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.value())
    }
}
