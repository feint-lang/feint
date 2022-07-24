//! Built in function type
use std::any::Any;
use std::fmt;

use crate::vm::{Chunk, RuntimeBoolResult, RuntimeContext, RuntimeErr, RuntimeResult};

use super::class::TypeRef;
use super::object::{Object, ObjectExt, ObjectRef};

pub struct Func {
    class: TypeRef,
    name: String,
    params: Vec<String>,
    pub chunk: Chunk,
}

impl Func {
    pub fn new<S: Into<String>>(
        class: TypeRef,
        name: S,
        params: Vec<String>,
        chunk: Chunk,
    ) -> Self {
        Self { class, name: name.into(), params, chunk }
    }
}

impl Object for Func {
    fn class(&self) -> &TypeRef {
        &self.class
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_equal(&self, rhs: &ObjectRef, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            Ok(self.is(&rhs))
        } else {
            Err(RuntimeErr::new_type_err(format!(
                "Could not compare {} to {}",
                self.class().name(),
                rhs.class().name()
            )))
        }
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
