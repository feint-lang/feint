use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::Code;

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Module Type ---------------------------------------------------------

gen::type_and_impls!(ModuleType, Module);

pub static MODULE_TYPE: Lazy<new::obj_ref_t!(ModuleType)> =
    Lazy::new(|| new::obj_ref!(ModuleType::new()));

// Module Object ----------------------------------------------------------

pub struct Module {
    name: String,
    ns: Namespace,
    pub code: Code,
}

gen::standard_object_impls!(Module);

impl Module {
    pub fn new(name: String, ns: Namespace, code: Code) -> Self {
        Self { ns, name, code }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn has_name(&self, name: &str) -> bool {
        self.ns.has(name)
    }
}

impl ObjectTrait for Module {
    gen::object_trait_header!(MODULE_TYPE);
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
