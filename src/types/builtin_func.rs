use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::vm::VM;

use super::create;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;
use super::result::{Args, CallResult, Params, This};

pub type BuiltinFn = fn(This, Args, &mut VM) -> CallResult;

// Builtin Function Type -----------------------------------------------

pub static BUILTIN_FUNC_TYPE: Lazy<Arc<BuiltinFuncType>> =
    Lazy::new(|| Arc::new(BuiltinFuncType::new()));

pub struct BuiltinFuncType {
    namespace: RefCell<Namespace>,
}

unsafe impl Send for BuiltinFuncType {}
unsafe impl Sync for BuiltinFuncType {}

impl BuiltinFuncType {
    pub fn new() -> Self {
        let mut ns = Namespace::new();
        ns.add_obj("$name", create::new_str("BuiltinFunc"));
        ns.add_obj("$full_name", create::new_str("builtins.BuiltinFunc"));
        Self { namespace: RefCell::new(ns) }
    }
}

impl TypeTrait for BuiltinFuncType {
    fn name(&self) -> &str {
        "BuiltinFunc"
    }

    fn full_name(&self) -> &str {
        "builtins.BuiltinFunc"
    }
}

impl ObjectTrait for BuiltinFuncType {
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

// BuiltinFunc Object ----------------------------------------------------------

pub struct BuiltinFunc {
    namespace: RefCell<Namespace>,
    pub name: String,
    pub params: Params,
    pub arity: Option<usize>,
    pub func: BuiltinFn,
}

unsafe impl Send for BuiltinFunc {}
unsafe impl Sync for BuiltinFunc {}

impl BuiltinFunc {
    pub fn new<S: Into<String>>(name: S, params: Params, func: BuiltinFn) -> Self {
        let arity = params.as_ref().map(|params| params.len());
        Self {
            namespace: RefCell::new(Namespace::new()),
            name: name.into(),
            params,
            arity,
            func,
        }
    }
}

impl ObjectTrait for BuiltinFunc {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        BUILTIN_FUNC_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        BUILTIN_FUNC_TYPE.clone()
    }

    fn namespace(&self) -> &RefCell<Namespace> {
        &self.namespace
    }

    fn call(&self, this: This, args: Args, vm: &mut VM) -> CallResult {
        (self.func)(this, args, vm)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for BuiltinFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = &self.name;
        let num_args = match self.arity {
            Some(n) => n.to_string(),
            None => "...".to_string(),
        };
        let id = self.id();
        write!(f, "builtin function {name}/{num_args} @ {id}")
    }
}

impl fmt::Debug for BuiltinFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
