use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::RuntimeBoolResult;

use super::create;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Nil Type ------------------------------------------------------------

pub static NIL_TYPE: Lazy<Arc<RwLock<NilType>>> =
    Lazy::new(|| Arc::new(RwLock::new(NilType::new())));

pub struct NilType {
    namespace: Namespace,
}

unsafe impl Send for NilType {}
unsafe impl Sync for NilType {}

impl NilType {
    pub fn new() -> Self {
        Self {
            namespace: Namespace::with_entries(&[
                // Class Attributes
                ("$name", create::new_str("Nil")),
                ("$full_name", create::new_str("builtins.Nil")),
            ]),
        }
    }
}

impl TypeTrait for NilType {
    fn name(&self) -> &str {
        "Nil"
    }

    fn full_name(&self) -> &str {
        "builtins.Nil"
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

impl ObjectTrait for NilType {
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

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

// Nil Object ----------------------------------------------------------

pub struct Nil {
    namespace: Namespace,
}

unsafe impl Send for Nil {}
unsafe impl Sync for Nil {}

impl Nil {
    pub fn new() -> Self {
        Self { namespace: Namespace::new() }
    }
}

impl ObjectTrait for Nil {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn class(&self) -> TypeRef {
        NIL_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        NIL_TYPE.clone()
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    fn bool_val(&self) -> RuntimeBoolResult {
        Ok(false)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Nil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "nil")
    }
}

impl fmt::Debug for Nil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
