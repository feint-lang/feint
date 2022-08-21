use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::Code;

use super::new;
use super::result::Params;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::func_trait::FuncTrait;
use super::ns::Namespace;

// Function Type -------------------------------------------------------

pub static FUNC_TYPE: Lazy<new::obj_ref_t!(FuncType)> =
    Lazy::new(|| new::obj_ref!(FuncType::new()));

pub struct FuncType {
    ns: Namespace,
}

unsafe impl Send for FuncType {}
unsafe impl Sync for FuncType {}

impl FuncType {
    pub fn new() -> Self {
        Self {
            ns: Namespace::with_entries(&[
                // Class Attributes
                ("$name", new::str("Func")),
                ("$full_name", new::str("builtins.Func")),
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

    fn ns(&self) -> &Namespace {
        &self.ns
    }
}

impl ObjectTrait for FuncType {
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

// Func Object ----------------------------------------------------------

pub struct Func {
    ns: Namespace,
    name: String,
    params: Params,
    pub code: Code,
}

unsafe impl Send for Func {}
unsafe impl Sync for Func {}

impl Func {
    pub fn new(name: String, params: Params, code: Code) -> Self {
        Self {
            ns: Namespace::with_entries(&[
                // Instance Attributes
                ("$name", new::str(name.as_str())),
            ]),
            name,
            params,
            code,
        }
    }

    pub fn arg_names(&self) -> Vec<&str> {
        let mut names = vec![];
        for name in self.params.iter() {
            if name.is_empty() {
                names.push("$args");
            } else {
                names.push(name);
            }
        }
        names
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

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn class(&self) -> TypeRef {
        FUNC_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        FUNC_TYPE.clone()
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
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
