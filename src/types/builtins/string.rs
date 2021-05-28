//! Built in string type
use std::any::Any;
use std::fmt;
use std::rc::Rc;

use crate::vm::{RuntimeBoolResult, RuntimeContext, RuntimeError, RuntimeResult};

use super::super::class::{Type, TypeRef};
use super::super::object::{Object, ObjectExt, ObjectRef};

pub type RustString = std::string::String;

#[derive(Debug, PartialEq)]
pub struct String {
    class: TypeRef,
    value: RustString,
}

impl String {
    pub fn new<S: Into<RustString>>(class: TypeRef, value: S) -> Self {
        Self { class: class.clone(), value: value.into() }
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

impl Object for String {
    fn class(&self) -> &TypeRef {
        &self.class
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_equal(&self, rhs: ObjectRef, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            Ok(self.is(rhs) || self.value() == rhs.value())
        } else {
            Err(RuntimeError::new_type_error(format!(
                "Could not compare String to {} for equality",
                rhs.class().name()
            )))
        }
    }

    fn add(&self, rhs: ObjectRef, ctx: &RuntimeContext) -> RuntimeResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            let a = self.value();
            let b = rhs.value();
            let mut value = RustString::with_capacity(a.len() + b.len());
            value.push_str(a);
            value.push_str(b);
            let value = ctx.builtins.new_string(value);
            Ok(value)
        } else {
            Err(RuntimeError::new_type_error(format!(
                "Could not concatenate String with {}",
                rhs.class().name()
            )))
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}
