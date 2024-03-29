//! Intrinsic (implemented in Rust) function type.
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::{Lazy, OnceCell};

use crate::modules::get_module;
use crate::vm::VM;

use super::gen;
use super::new;
use super::result::{Args, CallResult, Params};

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::func_trait::FuncTrait;
use super::ns::Namespace;

pub type IntrinsicFn = fn(ObjectRef, Args, &mut VM) -> CallResult;

// Intrinsic Function Type ---------------------------------------------

gen::type_and_impls!(IntrinsicFuncType, IntrinsicFunc);

pub static INTRINSIC_FUNC_TYPE: Lazy<gen::obj_ref_t!(IntrinsicFuncType)> =
    Lazy::new(|| gen::obj_ref!(IntrinsicFuncType::new()));

// IntrinsicFunc Object ------------------------------------------------

pub struct IntrinsicFunc {
    ns: Namespace,
    module_name: String,
    module: OnceCell<ObjectRef>,
    name: String,
    this_type: Option<ObjectRef>,
    params: Params,
    func: IntrinsicFn,
}

gen::standard_object_impls!(IntrinsicFunc);

impl IntrinsicFunc {
    pub fn new(
        module_name: String,
        name: String,
        this_type: Option<ObjectRef>,
        params: Params,
        doc: ObjectRef,
        func: IntrinsicFn,
    ) -> Self {
        Self {
            ns: Namespace::with_entries(&[
                // Instance Attributes
                ("$module_name", new::str(module_name.as_str())),
                ("$full_name", new::str(format!("{module_name}.{name}"))),
                ("$name", new::str(name.as_str())),
                ("$doc", doc),
            ]),
            module_name,
            module: OnceCell::default(),
            name,
            this_type,
            params,
            func,
        }
    }

    pub fn this_type(&self) -> Option<ObjectRef> {
        self.this_type.clone()
    }

    pub fn func(&self) -> &IntrinsicFn {
        &self.func
    }
}

impl FuncTrait for IntrinsicFunc {
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

impl ObjectTrait for IntrinsicFunc {
    gen::object_trait_header!(INTRINSIC_FUNC_TYPE);

    fn module(&self) -> ObjectRef {
        self.module.get_or_init(|| get_module(&self.module_name)).clone()
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for IntrinsicFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", FuncTrait::format_string(self, Some(self.id())))
    }
}

impl fmt::Debug for IntrinsicFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
