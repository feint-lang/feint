use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::VM;

use super::new;
use super::result::{Args, CallResult, Params, This};

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::func_trait::FuncTrait;
use super::ns::Namespace;

pub type BuiltinFn = fn(This, Args, &mut VM) -> CallResult;

// Builtin Function Type -----------------------------------------------

pub static BUILTIN_FUNC_TYPE: Lazy<Arc<RwLock<BuiltinFuncType>>> =
    Lazy::new(|| Arc::new(RwLock::new(BuiltinFuncType::new())));

pub struct BuiltinFuncType {
    ns: Namespace,
}

unsafe impl Send for BuiltinFuncType {}
unsafe impl Sync for BuiltinFuncType {}

impl BuiltinFuncType {
    pub fn new() -> Self {
        Self {
            ns: Namespace::with_entries(&[
                // Class Attributes
                ("$name", new::str("BuiltinFunc")),
                ("$full_name", new::str("builtins.BuiltinFunc")),
            ]),
        }
    }
}

impl TypeTrait for BuiltinFuncType {
    fn name(&self) -> &str {
        "BuiltinFunc"
    }

    fn full_name(&self) -> &str {
        "builtins.BuiltinFunc"
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }
}

impl ObjectTrait for BuiltinFuncType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        TYPE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        TYPE_TYPE.clone()
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
    }
}

// BuiltinFunc Object ----------------------------------------------------------

pub struct BuiltinFunc {
    ns: Namespace,
    name: String,
    params: Params,
    pub func: BuiltinFn,
}

unsafe impl Send for BuiltinFunc {}
unsafe impl Sync for BuiltinFunc {}

impl BuiltinFunc {
    pub fn new(name: String, params: Params, func: BuiltinFn) -> Self {
        Self {
            ns: Namespace::with_entries(&[
                // Instance Attributes
                ("$name", new::str(name.as_str())),
            ]),
            name,
            params,
            func,
        }
    }
}

impl FuncTrait for BuiltinFunc {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn params(&self) -> &Params {
        &self.params
    }
}

impl ObjectTrait for BuiltinFunc {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        BUILTIN_FUNC_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        BUILTIN_FUNC_TYPE.clone()
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for BuiltinFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", FuncTrait::format_string(self, Some(self.id())))
    }
}

impl fmt::Debug for BuiltinFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
