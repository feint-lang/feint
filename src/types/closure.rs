use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::{RuntimeResult, VM};

use super::create;
use super::result::Args;
use super::util::args_to_str;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Closure Type --------------------------------------------------------

pub static CLOSURE_TYPE: Lazy<Arc<RwLock<ClosureType>>> =
    Lazy::new(|| Arc::new(RwLock::new(ClosureType::new())));

pub struct ClosureType {
    namespace: Namespace,
}

unsafe impl Send for ClosureType {}
unsafe impl Sync for ClosureType {}

impl ClosureType {
    pub fn new() -> Self {
        Self {
            namespace: Namespace::with_entries(&[
                // Class Attributes
                ("$name", create::new_str("Closure")),
                ("$full_name", create::new_str("builtins.Closure")),
            ]),
        }
    }
}

impl TypeTrait for ClosureType {
    fn name(&self) -> &str {
        "Closure"
    }

    fn full_name(&self) -> &str {
        "builtins.Closure"
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

impl ObjectTrait for ClosureType {
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

// Closure Object ------------------------------------------------------

pub struct Closure {
    namespace: Namespace,
    pub func: ObjectRef,
    pub captured: Vec<usize>,
}

unsafe impl Send for Closure {}
unsafe impl Sync for Closure {}

impl Closure {
    pub fn new(func_ref: ObjectRef, captured: Vec<usize>) -> Self {
        Self { namespace: Namespace::new(), func: func_ref, captured }
    }
}

impl ObjectTrait for Closure {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        CLOSURE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        CLOSURE_TYPE.clone()
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    fn call(&self, args: Args, vm: &mut VM) -> RuntimeResult {
        log::trace!("BEGIN: call closure {self}");
        log::trace!("ARGS: {}", args_to_str(&args));
        vm.call_closure(self, args)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[closure] {}", self.func.read().unwrap())
    }
}

impl fmt::Debug for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
