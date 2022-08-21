use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::Code;

use super::gen;
use super::new;
use super::result::Params;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::func_trait::FuncTrait;
use super::ns::Namespace;

// Function Type -------------------------------------------------------

gen::type_and_impls!(FuncType, Func);

pub static FUNC_TYPE: Lazy<new::obj_ref_t!(FuncType)> =
    Lazy::new(|| new::obj_ref!(FuncType::new()));

// Func Object ----------------------------------------------------------

pub struct Func {
    ns: Namespace,
    name: String,
    params: Params,
    pub code: Code,
}

gen::standard_object_impls!(Func);

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
    gen::object_trait_header!(FUNC_TYPE);
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
