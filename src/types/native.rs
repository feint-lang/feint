//! Built in native function type
use std::any::Any;
use std::fmt;

use crate::vm::{Chunk, RuntimeBoolResult, RuntimeContext, RuntimeErr, RuntimeResult};

use super::class::TypeRef;
use super::object::{Object, ObjectExt, ObjectRef};
use super::result::CallResult;

pub type NativeFn = fn(Vec<ObjectRef>, &RuntimeContext) -> Option<ObjectRef>;

pub struct NativeFunc {
    class: TypeRef,
    name: String,
    func: NativeFn,
    arg_count: Option<u8>,
}

impl NativeFunc {
    pub fn new<S: Into<String>>(
        class: TypeRef,
        name: S,
        func: NativeFn,
        arg_count: Option<u8>,
    ) -> Self {
        Self { class, name: name.into(), func, arg_count }
    }
}

impl Object for NativeFunc {
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

    fn call(&self, args: Vec<ObjectRef>, ctx: &RuntimeContext) -> CallResult {
        Ok((self.func)(args, ctx))
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for NativeFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let num_args = match self.arg_count {
            Some(n) => n.to_string(),
            None => "...".to_string(),
        };
        write!(f, "{} ({}) ->", self.name, num_args)
    }
}

impl fmt::Debug for NativeFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
