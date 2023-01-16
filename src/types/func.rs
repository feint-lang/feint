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

pub static FUNC_TYPE: Lazy<gen::obj_ref_t!(FuncType)> =
    Lazy::new(|| gen::obj_ref!(FuncType::new()));

// Func Object ----------------------------------------------------------

pub struct Func {
    ns: Namespace,
    name: String,
    params: Params,
    code: Code,
}

gen::standard_object_impls!(Func);

impl Func {
    pub fn new(name: String, params: Params, code: Code) -> Self {
        Self {
            ns: Namespace::with_entries(&[
                // Instance Attributes
                ("$name", new::str(name.as_str())),
                ("$doc", code.get_doc()),
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

    pub fn code(&self) -> &Code {
        &self.code
    }
}

impl FuncTrait for Func {
    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn params(&self) -> &Params {
        &self.params
    }
}

impl ObjectTrait for Func {
    gen::object_trait_header!(FUNC_TYPE);

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if self.is(rhs) || rhs.is_always() {
            true
        } else if let Some(f) = rhs.down_to_func() {
            f.params == self.params && f.code == self.code
        } else {
            false
        }
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
