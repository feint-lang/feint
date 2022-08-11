use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::Code;

use super::create;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Module Type ---------------------------------------------------------

pub static MODULE_TYPE: Lazy<Arc<RwLock<ModuleType>>> =
    Lazy::new(|| Arc::new(RwLock::new(ModuleType::new())));

pub struct ModuleType {
    namespace: Namespace,
}

unsafe impl Send for ModuleType {}
unsafe impl Sync for ModuleType {}

impl ModuleType {
    pub fn new() -> Self {
        Self {
            namespace: Namespace::with_entries(&[
                // Class Attributes
                ("$name", create::new_str("Module")),
                ("$full_name", create::new_str("builtins.Module")),
            ]),
        }
    }
}

impl TypeTrait for ModuleType {
    fn name(&self) -> &str {
        "Module"
    }

    fn full_name(&self) -> &str {
        "builtins.Module"
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

impl ObjectTrait for ModuleType {
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

// Module Object ----------------------------------------------------------

pub struct Module {
    name: String,
    namespace: Namespace,
    pub code: Code,
}

unsafe impl Send for Module {}
unsafe impl Sync for Module {}

impl Module {
    pub fn new(name: String, namespace: Namespace, code: Code) -> Self {
        Self { namespace, name, code }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl ObjectTrait for Module {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        MODULE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        MODULE_TYPE.clone()
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<module {}>", self.name())
    }
}

impl fmt::Debug for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<module {} @ {}>", self.name(), self.id())
    }
}
