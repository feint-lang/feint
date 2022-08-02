//! Function type
use std::any::Any;
use std::fmt;

use crate::vm::{Chunk, VM};

use super::builtin_types::BUILTIN_TYPES;
use super::class::TypeRef;
use super::object::{Object, ObjectRef};
use super::result::{Args, CallResult, Params};

pub struct Func {
    pub name: String,
    pub params: Params,
    pub arity: Option<usize>,
    pub chunk: Chunk,
    pub this: Option<ObjectRef>,
}

impl Func {
    pub fn new<S: Into<String>>(
        name: S,
        params: Params,
        chunk: Chunk,
        this: Option<ObjectRef>,
    ) -> Self {
        let arity = params.as_ref().map(|params| params.len());
        Self { name: name.into(), params, arity, chunk, this }
    }
}

impl Object for Func {
    fn class(&self) -> &TypeRef {
        BUILTIN_TYPES.get("Func").unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    /// This provides a way to call user functions from builtin
    /// functions. Perhaps there's a better way to do this?
    fn call(&self, args: Args, vm: &mut VM) -> CallResult {
        vm.scope_stack.push(vm.value_stack.size());
        vm.ctx.enter_scope();
        vm.check_call_args(self.name.as_str(), &self.params, &args, true)?;
        let result = if let Err(err) = vm.execute(&self.chunk, false) {
            Err(err)
        } else {
            let result = vm.pop_obj()?;
            Ok(Some(result))
        };
        vm.ctx.exit_scopes(1);
        result
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = &self.name;
        let num_args = match self.arity {
            Some(n) => n.to_string(),
            None => "...".to_string(),
        };
        let id = self.id();
        write!(f, "function {name}/{num_args} @ {id}")
    }
}

impl fmt::Debug for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
