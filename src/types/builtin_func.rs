use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::VM;

use super::gen;
use super::new;
use super::result::{Args, CallResult, Params};

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::func_trait::FuncTrait;
use super::ns::Namespace;

pub type BuiltinFn = fn(ObjectRef, Args, &mut VM) -> CallResult;

// Builtin Function Type -----------------------------------------------

gen::type_and_impls!(BuiltinFuncType, BuiltinFunc);

pub static BUILTIN_FUNC_TYPE: Lazy<new::obj_ref_t!(BuiltinFuncType)> =
    Lazy::new(|| new::obj_ref!(BuiltinFuncType::new()));

// BuiltinFunc Object ----------------------------------------------------------

pub struct BuiltinFunc {
    ns: Namespace,
    name: String,
    this_type: Option<ObjectRef>,
    params: Params,
    pub func: BuiltinFn,
}

gen::standard_object_impls!(BuiltinFunc);

impl BuiltinFunc {
    pub fn new(
        name: String,
        this_type: Option<ObjectRef>,
        params: Params,
        func: BuiltinFn,
    ) -> Self {
        Self {
            ns: Namespace::with_entries(&[
                // Instance Attributes
                ("$name", new::str(name.as_str())),
            ]),
            name,
            this_type,
            params,
            func,
        }
    }

    pub fn this_type(&self) -> Option<ObjectRef> {
        self.this_type.clone()
    }
}

impl FuncTrait for BuiltinFunc {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn params(&self) -> &Params {
        &self.params
    }
}

impl ObjectTrait for BuiltinFunc {
    gen::object_trait_header!(BUILTIN_FUNC_TYPE);
}

// Display -------------------------------------------------------------

impl fmt::Display for BuiltinFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", FuncTrait::format_string(self, Some(self.id())))
    }
}

impl fmt::Debug for BuiltinFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
