use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use feint_code_gen::*;

use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::func_trait::FuncTrait;
use super::ns::Namespace;
use super::Params;

// Bound Function Type -------------------------------------------------

type_and_impls!(BoundFuncType, BoundFunc);

pub static BOUND_FUNC_TYPE: Lazy<obj_ref_t!(BoundFuncType)> =
    Lazy::new(|| obj_ref!(BoundFuncType::new()));

// BoundFunc Object ----------------------------------------------------------

pub struct BoundFunc {
    ns: Namespace,
    module_name: String,
    func: ObjectRef,
    this: ObjectRef,
    name: String,
    params: Params,
}

standard_object_impls!(BoundFunc);

impl BoundFunc {
    pub fn new(func_ref: ObjectRef, this: ObjectRef) -> Self {
        let (module_name, name, doc, params, params_tuple, arity, has_var_args) = {
            let func_guard = func_ref.read().unwrap();

            let func: &dyn FuncTrait =
                if let Some(func) = func_guard.down_to_intrinsic_func() {
                    func
                } else if let Some(func) = func_guard.down_to_func() {
                    func
                } else if let Some(func) = func_guard.down_to_closure() {
                    func
                } else {
                    panic!("Unexpected bound func type: {func_guard}")
                };

            (
                func.module_name().to_owned(),
                func.name().to_owned(),
                func.get_doc(),
                func.params().clone(),
                func.get_params(),
                func.arity(),
                func.has_var_args(),
            )
        };

        Self {
            ns: Namespace::with_entries(&[
                ("$module_name", new::str(&module_name)),
                ("$full_name", new::str(format!("{module_name}.{name}"))),
                ("$name", new::str(&name)),
                ("$params", params_tuple),
                ("$doc", doc),
                ("$arity", new::int(arity)),
                ("$has_var_args", new::bool(has_var_args)),
            ]),
            module_name,
            func: func_ref,
            this,
            name,
            params,
        }
    }

    pub fn func(&self) -> ObjectRef {
        self.func.clone()
    }

    pub fn this(&self) -> ObjectRef {
        self.this.clone()
    }
}

impl FuncTrait for BoundFunc {
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

impl ObjectTrait for BoundFunc {
    object_trait_header!(BOUND_FUNC_TYPE);

    fn module(&self) -> ObjectRef {
        self.func().read().unwrap().module()
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for BoundFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.func.read().unwrap())
    }
}

impl fmt::Debug for BoundFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} *BOUND* to {:?}",
            &*self.func.read().unwrap(),
            &*self.this.read().unwrap()
        )
    }
}
