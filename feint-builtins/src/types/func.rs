use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::{Lazy, OnceCell};

use feint_code_gen::*;

use crate::modules::get_module;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::code::Code;
use super::func_trait::FuncTrait;
use super::ns::Namespace;
use super::{new, Params};

// Function Type -------------------------------------------------------

type_and_impls!(FuncType, Func);

pub static FUNC_TYPE: Lazy<obj_ref_t!(FuncType)> =
    Lazy::new(|| obj_ref!(FuncType::new()));

// Func Object ----------------------------------------------------------

pub struct Func {
    ns: Namespace,
    module_name: String,
    module: OnceCell<ObjectRef>,
    name: String,
    params: Params,
    code: Code,
}

standard_object_impls!(Func);

impl Func {
    pub fn new(module_name: String, name: String, params: Params, code: Code) -> Self {
        let params_tuple = new::tuple(params.iter().map(new::str).collect());

        let mut instance = Self {
            ns: Namespace::with_entries(&[
                // Instance Attributes
                ("$module_name", new::str(&module_name)),
                ("$full_name", new::str(format!("{module_name}.{name}"))),
                ("$name", new::str(&name)),
                ("$params", params_tuple),
                ("$doc", code.get_doc()),
            ]),
            module_name,
            module: OnceCell::default(),
            name,
            params,
            code,
        };

        let arity = (&instance as &dyn FuncTrait).arity();
        let has_var_args = (&instance as &dyn FuncTrait).has_var_args();
        instance.ns_mut().insert("$arity", new::int(arity));
        instance.ns_mut().insert("$has_var_args", new::bool(has_var_args));

        instance
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

    fn module_name(&self) -> &String {
        &self.module_name
    }

    fn module(&self) -> ObjectRef {
        (self as &dyn ObjectTrait).module()
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn params(&self) -> &Params {
        &self.params
    }
}

impl ObjectTrait for Func {
    object_trait_header!(FUNC_TYPE);

    fn module(&self) -> ObjectRef {
        self.module.get_or_init(|| get_module(&self.module_name)).clone()
    }

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
        write!(f, "{}", FuncTrait::format_string(self, None))
    }
}

impl fmt::Debug for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", FuncTrait::format_string(self, Some(self.id())))
    }
}
