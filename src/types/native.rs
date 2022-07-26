//! Built in native function type
use std::any::Any;
use std::fmt;

use crate::vm::RuntimeContext;

use super::class::Type;
use super::object::{Object, ObjectRef};
use super::result::CallResult;
use super::types::TYPES;

pub type NativeFn = fn(Vec<ObjectRef>, &RuntimeContext) -> CallResult;

pub struct NativeFunc {
    pub name: String,
    func: NativeFn,
    pub arity: Option<u8>,
}

impl NativeFunc {
    pub fn new<S: Into<String>>(name: S, func: NativeFn, arity: Option<u8>) -> Self {
        Self { name: name.into(), func, arity }
    }
}

impl Object for NativeFunc {
    fn class(&self) -> &Type {
        TYPES.get("NativeFunc").unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn call(&self, args: Vec<ObjectRef>, ctx: &RuntimeContext) -> CallResult {
        (self.func)(args, ctx)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for NativeFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let num_args = match self.arity {
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
