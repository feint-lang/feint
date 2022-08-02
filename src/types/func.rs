use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::vm::{Chunk, VM};

use super::create;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;
use super::result::{Args, CallResult, Params, This};

// Function Type -------------------------------------------------------

pub static FUNC_TYPE: Lazy<Arc<FuncType>> = Lazy::new(|| Arc::new(FuncType::new()));

pub struct FuncType {
    namespace: RefCell<Namespace>,
}

unsafe impl Send for FuncType {}
unsafe impl Sync for FuncType {}

impl FuncType {
    pub fn new() -> Self {
        let mut ns = Namespace::new();
        ns.add_obj("$name", create::new_str("Func"));
        ns.add_obj("$full_name", create::new_str("builtins.Func"));
        Self { namespace: RefCell::new(ns) }
    }
}

impl TypeTrait for FuncType {
    fn name(&self) -> &str {
        "Func"
    }

    fn full_name(&self) -> &str {
        "builtins.Func"
    }
}

impl ObjectTrait for FuncType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        TYPE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        TYPE_TYPE.clone()
    }

    fn namespace(&self) -> &RefCell<Namespace> {
        &self.namespace
    }
}

// Func Object ----------------------------------------------------------

pub struct Func {
    namespace: RefCell<Namespace>,
    pub name: String,
    pub params: Params,
    pub arity: Option<usize>,
    pub chunk: Chunk,
}

unsafe impl Send for Func {}
unsafe impl Sync for Func {}

impl Func {
    pub fn new<S: Into<String>>(name: S, params: Params, chunk: Chunk) -> Self {
        let arity = params.as_ref().map(|params| params.len());
        Self {
            namespace: RefCell::new(Namespace::new()),
            name: name.into(),
            params,
            arity,
            chunk,
        }
    }
}

impl ObjectTrait for Func {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        FUNC_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        FUNC_TYPE.clone()
    }

    fn namespace(&self) -> &RefCell<Namespace> {
        &self.namespace
    }

    /// This provides a way to call user functions from builtin
    /// functions. Perhaps there's a better way to do this?
    fn call(&self, this: This, args: Args, vm: &mut VM) -> CallResult {
        vm.enter_scope();
        if let Some(this) = &this {
            vm.ctx.declare_and_assign_var("this", this.clone())?;
        }
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
