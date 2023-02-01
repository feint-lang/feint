//! Intrinsic (implemented in Rust) function type.
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::{Lazy, OnceCell};

use feint_code_gen::*;

use crate::modules::get_module;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::func_trait::FuncTrait;
use super::ns::Namespace;
use super::{new, Args, CallResult, Params};

pub type IntrinsicFn = fn(ObjectRef, Args) -> CallResult;

// Intrinsic Function Type ---------------------------------------------

type_and_impls!(IntrinsicFuncType, IntrinsicFunc);

pub static INTRINSIC_FUNC_TYPE: Lazy<obj_ref_t!(IntrinsicFuncType)> =
    Lazy::new(|| obj_ref!(IntrinsicFuncType::new()));

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

standard_object_impls!(IntrinsicFunc);

impl IntrinsicFunc {
    pub fn new(
        module_name: String,
        name: String,
        this_type: Option<ObjectRef>,
        params: Params,
        doc: ObjectRef,
        func: IntrinsicFn,
    ) -> Self {
        let params_tuple = new::tuple(params.iter().map(new::str).collect());

        let mut instance = Self {
            ns: Namespace::with_entries(&[
                // Instance Attributes
                ("$module_name", new::str(module_name.as_str())),
                ("$full_name", new::str(format!("{module_name}.{name}"))),
                ("$name", new::str(name.as_str())),
                ("$params", params_tuple),
                ("$doc", doc),
            ]),
            module_name,
            module: OnceCell::default(),
            name,
            this_type,
            params,
            func,
        };

        let arity = (&instance as &dyn FuncTrait).arity();
        let has_var_args = (&instance as &dyn FuncTrait).has_var_args();
        instance.ns_mut().insert("$arity", new::int(arity));
        instance.ns_mut().insert("$has_var_args", new::bool(has_var_args));

        instance
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
    object_trait_header!(INTRINSIC_FUNC_TYPE);

    fn module(&self) -> ObjectRef {
        self.module.get_or_init(|| get_module(&self.module_name)).clone()
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for IntrinsicFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", FuncTrait::format_string(self, None))
    }
}

impl fmt::Debug for IntrinsicFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", FuncTrait::format_string(self, Some(self.id())))
    }
}
