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
    code: Code,
}

gen::standard_object_impls!(Module);

impl Module {
    /// NOTE: The `$doc` attribute should only be passed for builtin
    ///       modules implemented in Rust and for special cases such as
    ///       the REPL module. Modules implemented in FeInt will have
    ///       their `$doc` attribute initialized from their module level
    ///       docstring.
    pub fn new(name: String, mut ns: Namespace, code: Code, doc: Option<&str>) -> Self {
        ns.add_obj("$name", new::str(name.as_str()));
        if let Some(doc) = doc {
            ns.add_obj("$doc", new::str(doc));
        } else {
            let doc = code.get_doc();
            ns.add_obj("$doc", doc);
        }
        Self { ns, name, code }
    }

    pub fn with_name(name: &str, doc: Option<&str>) -> Self {
        Self::new(name.to_owned(), Namespace::new(), Code::new(), doc)
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn add_global(&mut self, name: &str, val: ObjectRef) {
        self.ns.add_obj(name, val.clone());
    }

    pub fn get_global(&self, name: &str) -> Option<ObjectRef> {
        self.ns.get_obj(name)
    }

    pub fn has_global(&self, name: &str) -> bool {
        self.ns.has(name)
    }

    pub fn iter_globals(&self) -> indexmap::map::Iter<'_, String, ObjectRef> {
        self.ns.iter()
    }

    pub fn code(&self) -> &Code {
        &self.code
    }

    pub fn code_mut(&mut self) -> &mut Code {
        &mut self.code
    }

    pub fn set_code(&mut self, code: Code) {
        self.code = code;
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
