//! Builtin function type
use std::any::Any;
use std::fmt;

use crate::vm::VM;

use super::builtin_types::BUILTIN_TYPES;
use super::class::TypeRef;
use super::object::Object;
use super::result::{Args, CallResult, Params};

pub type BuiltinFn = fn(Args, &mut VM) -> CallResult;

pub struct BuiltinFunc {
    pub name: String,
    pub params: Params,
    pub arity: Option<usize>,
    pub func: BuiltinFn,
}

impl BuiltinFunc {
    pub fn new<S: Into<String>>(name: S, params: Params, func: BuiltinFn) -> Self {
        let arity = if let Some(params) = &params { Some(params.len()) } else { None };
        Self { name: name.into(), params, arity, func }
    }
}

impl Object for BuiltinFunc {
    fn class(&self) -> &TypeRef {
        BUILTIN_TYPES.get("BuiltinFunc").unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn call(&self, args: Args, vm: &mut VM) -> CallResult {
        (self.func)(args, vm)
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
