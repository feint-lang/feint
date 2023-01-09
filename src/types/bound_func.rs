use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::func_trait::FuncTrait;
use super::ns::Namespace;
use super::result::Params;

// Bound Function Type -------------------------------------------------

gen::type_and_impls!(BoundFuncType, BoundFunc);

pub static BOUND_FUNC_TYPE: Lazy<new::obj_ref_t!(BoundFuncType)> =
    Lazy::new(|| new::obj_ref!(BoundFuncType::new()));

// BoundFunc Object ----------------------------------------------------------

pub struct BoundFunc {
    ns: Namespace,
    func: ObjectRef,
    this: ObjectRef,
    name: String,
    params: Params,
}

gen::standard_object_impls!(BoundFunc);

impl BoundFunc {
    pub fn new(func: ObjectRef, this: ObjectRef) -> Self {
        let f = func.read().unwrap();
        let (name, params, doc) = if let Some(f) = f.down_to_builtin_func() {
            (f.name().to_string(), f.params().clone(), f.get_doc())
        } else if let Some(f) = f.down_to_func() {
            (f.name().to_string(), f.params().clone(), f.get_doc())
        } else if let Some(f) = f.down_to_closure() {
            (f.name().to_string(), f.params().clone(), f.get_doc())
        } else {
            panic!("Unexpected bound func type: {f}")
        };
        drop(f);
        Self { ns: Namespace::with_entries(&[("$doc", doc)]), func, this, name, params }
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

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn params(&self) -> &Params {
        &self.params
    }
}

impl ObjectTrait for BoundFunc {
    gen::object_trait_header!(BOUND_FUNC_TYPE);
}

// Display -------------------------------------------------------------

impl fmt::Display for BoundFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} *BOUND* to {:?}",
            self.func.read().unwrap(),
            &*self.this.read().unwrap()
        )
    }
}

impl fmt::Debug for BoundFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
