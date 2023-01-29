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

pub static BOUND_FUNC_TYPE: Lazy<gen::obj_ref_t!(BoundFuncType)> =
    Lazy::new(|| gen::obj_ref!(BoundFuncType::new()));

// BoundFunc Object ----------------------------------------------------------

pub struct BoundFunc {
    ns: Namespace,
    module_name: String,
    func: ObjectRef,
    this: ObjectRef,
    name: String,
    params: Params,
}

gen::standard_object_impls!(BoundFunc);

impl BoundFunc {
    pub fn new(func: ObjectRef, this: ObjectRef) -> Self {
        let f = func.read().unwrap();

        let (module_name, name, params, doc) =
            if let Some(f) = f.down_to_intrinsic_func() {
                (f.module_name(), f.name(), f.params(), f.get_doc())
            } else if let Some(f) = f.down_to_func() {
                (f.module_name(), f.name(), f.params(), f.get_doc())
            } else if let Some(f) = f.down_to_closure() {
                (f.module_name(), f.name(), f.params(), f.get_doc())
            } else {
                panic!("Unexpected bound func type: {f}")
            };

        let module_name = module_name.to_owned();
        let name = name.to_owned();
        let params = params.clone();

        drop(f);

        Self {
            ns: Namespace::with_entries(&[
                ("$module_name", new::str(&module_name)),
                ("$full_name", new::str(format!("{module_name}.{name}"))),
                ("$name", new::str(&name)),
                ("$doc", doc),
            ]),
            module_name,
            func,
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
    gen::object_trait_header!(BOUND_FUNC_TYPE);

    fn module(&self) -> ObjectRef {
        self.func().read().unwrap().module()
    }
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
