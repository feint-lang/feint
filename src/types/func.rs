use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::{Code, RuntimeResult, VM};

use super::create;
use super::result::{Args, Params};
use super::util::args_to_str;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::func_trait::FuncTrait;
use super::ns::Namespace;

// Function Type -------------------------------------------------------

pub static FUNC_TYPE: Lazy<Arc<RwLock<FuncType>>> =
    Lazy::new(|| Arc::new(RwLock::new(FuncType::new())));

pub struct FuncType {
    namespace: Namespace,
}

unsafe impl Send for FuncType {}
unsafe impl Sync for FuncType {}

impl FuncType {
    pub fn new() -> Self {
        Self {
            namespace: Namespace::with_entries(&[
                // Class Attributes
                ("$name", create::new_str("Func")),
                ("$full_name", create::new_str("builtins.Func")),
            ]),
        }
    }
}

impl TypeTrait for FuncType {
    fn name(&self) -> &str {
        "Func"
    }

    fn full_name(&self) -> &str {
        "builtins.Func"
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
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

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

// Func Object ----------------------------------------------------------

pub struct Func {
    namespace: Namespace,
    name: String,
    params: Params,
    pub code: Code,
}

unsafe impl Send for Func {}
unsafe impl Sync for Func {}

impl Func {
    pub fn new(name: String, params: Params, code: Code) -> Self {
        Self {
            namespace: Namespace::with_entries(&[
                // Instance Attributes
                ("$name", create::new_str(name.as_str())),
            ]),
            name,
            params,
            code,
        }
    }
}

impl FuncTrait for Func {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn params(&self) -> &Params {
        &self.params
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

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    fn call(&self, args: Args, vm: &mut VM) -> RuntimeResult {
        log::trace!("BEGIN: call {self}");
        log::trace!("ARGS: {}", args_to_str(&args));
        vm.call_func(self, None, args)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", FuncTrait::format_string(self, Some(self.id())))
    }
}

impl fmt::Debug for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
