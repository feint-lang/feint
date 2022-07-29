//! Function type
use std::any::Any;
use std::fmt;

use crate::types::{Args, CallResult, Params};
use crate::vm::{Chunk, RuntimeErr, VM};

use super::builtin_types::BUILTIN_TYPES;
use super::class::TypeRef;
use super::object::Object;

pub struct Func {
    pub name: String,
    pub params: Params,
    pub arity: Option<usize>,
    pub chunk: Chunk,
}

impl Func {
    pub fn new<S: Into<String>>(name: S, params: Params, chunk: Chunk) -> Self {
        let arity = if let Some(params) = &params { Some(params.len()) } else { None };
        Self { name: name.into(), params, arity, chunk }
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
    ///
    /// TODO: This duplicates the VM's handling of calls. The only
    ///       difference is that it returns the result rather than
    ///       pushing it back onto the stack as a return value.
    fn call(&self, args: Args, vm: &mut VM) -> CallResult {
        // Wrap the function call in a scope where the
        // function's locals are defined. After the
        // call, this scope will be cleared out.
        vm.scope_stack.push(vm.value_stack.size());
        vm.ctx.enter_scope();

        if let Some(params) = &self.params {
            let arity = params.len();
            let num_args = args.len();
            if num_args != arity {
                let name = &self.name;
                let ess = if arity == 1 { "" } else { "s" };
                return Err(RuntimeErr::new_type_err(format!(
                    "{name}() expected {arity} arg{ess}; got {num_args}"
                )));
            }
            // Bind args
            for (name, arg) in params.iter().zip(args) {
                vm.ctx.declare_and_assign_var(name, arg)?;
            }
        } else {
            let args = vm.ctx.builtins.new_tuple(args);
            vm.ctx.declare_and_assign_var("$args", args)?;
        }

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
        write!(f, "function {name}/{num_args}")
    }
}

impl fmt::Debug for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.id();
        write!(f, "{self} @ {id}")
    }
}
