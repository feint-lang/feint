//! Built in string type
use std::any::Any;
use std::fmt;

use crate::vm::{RuntimeBoolResult, RuntimeContext, RuntimeErr, RuntimeResult};

use crate::types::class::TypeRef;
use crate::types::object::{Object, ObjectExt, ObjectRef};

pub struct Str {
    class: TypeRef,
    value: String,
}

impl Str {
    pub fn new<S: Into<String>>(class: TypeRef, value: S) -> Self {
        Self { class, value: value.into() }
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

impl Object for Str {
    fn class(&self) -> &TypeRef {
        &self.class
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_equal(&self, rhs: &ObjectRef, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            Ok(self.is(rhs) || self.value() == rhs.value())
        } else {
            Err(RuntimeErr::new_type_error(format!(
                "Could not compare String to {} for equality",
                rhs.class().name()
            )))
        }
    }

    fn add(&self, rhs: &ObjectRef, ctx: &RuntimeContext) -> RuntimeResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            let a = self.value();
            let b = rhs.value();
            let mut value = String::with_capacity(a.len() + b.len());
            value.push_str(a);
            value.push_str(b);
            let value = ctx.builtins.new_string(value);
            Ok(value)
        } else {
            Err(RuntimeErr::new_type_error(format!(
                "Could not concatenate String with {}",
                rhs.class().name()
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
