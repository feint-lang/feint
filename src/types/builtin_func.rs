//! Builtin function type
use std::any::Any;
use std::fmt;

use crate::vm::RuntimeContext;

use super::builtin_types::BUILTIN_TYPES;
use super::class::TypeRef;
use super::object::{Object, ObjectRef};
use super::result::CallResult;

pub type BuiltinFn = fn(Vec<ObjectRef>, &RuntimeContext) -> CallResult;

pub struct BuiltinFunc {
    pub name: String,
    func: BuiltinFn,
    pub arity: Option<u8>,
}

impl BuiltinFunc {
    pub fn new<S: Into<String>>(name: S, func: BuiltinFn, arity: Option<u8>) -> Self {
        Self { name: name.into(), func, arity }
    }
}

impl Object for BuiltinFunc {
    fn class(&self) -> &TypeRef {
        BUILTIN_TYPES.get("BuiltinFunc").unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn call(&self, args: Vec<ObjectRef>, ctx: &RuntimeContext) -> CallResult {
        (self.func)(args, ctx)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for BuiltinFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let num_args = match self.arity {
            Some(n) => n.to_string(),
            None => "...".to_string(),
        };
        write!(f, "{} ({}) ->", self.name, num_args)
    }
}

impl fmt::Debug for BuiltinFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
