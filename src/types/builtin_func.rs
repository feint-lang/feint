//! Builtin function type
use std::any::Any;
use std::fmt;

use crate::vm::RuntimeContext;

use super::builtin_types::BUILTIN_TYPES;
use super::class::TypeRef;
use super::object::Object;
use super::result::{Args, CallResult};

pub type BuiltinFn = fn(Args, &RuntimeContext) -> CallResult;

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

    fn call(&self, args: Args, ctx: &RuntimeContext) -> CallResult {
        (self.func)(args, ctx)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for BuiltinFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = &self.name;
        let num_args = match self.arity {
            Some(0) => "".to_string(),
            Some(n) => n.to_string(),
            None => "...".to_string(),
        };
        let id = self.id();
        write!(f, "Builtin function {name} ({num_args}) @ {id}")
    }
}

impl fmt::Debug for BuiltinFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
