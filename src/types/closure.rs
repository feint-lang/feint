use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use once_cell::sync::Lazy;

use super::gen;
use super::new;
use super::result::Params;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::func_trait::FuncTrait;
use super::ns::Namespace;

// Closure Type --------------------------------------------------------

gen::type_and_impls!(ClosureType, Closure);

pub static CLOSURE_TYPE: Lazy<new::obj_ref_t!(ClosureType)> =
    Lazy::new(|| new::obj_ref!(ClosureType::new()));

// Closure Object ------------------------------------------------------

pub struct Closure {
    ns: Namespace,
    name: String,
    params: Params,
    pub func: ObjectRef,
    pub captured: IndexMap<String, ObjectRef>,
}

gen::standard_object_impls!(Closure);

impl Closure {
    pub fn new(func_ref: ObjectRef, captured: IndexMap<String, ObjectRef>) -> Self {
        let func = func_ref.read().unwrap();
        let func = func.down_to_func().unwrap();
        Self {
            ns: Namespace::new(),
            name: func.name().to_owned(),
            params: func.params().clone(),
            func: func_ref.clone(),
            captured,
        }
    }
}

impl FuncTrait for Closure {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn params(&self) -> &Params {
        &self.params
    }
}

impl ObjectTrait for Closure {
    gen::object_trait_header!(CLOSURE_TYPE);
}

// Display -------------------------------------------------------------

impl fmt::Display for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[closure] {}", self.func.read().unwrap())
    }
}

impl fmt::Debug for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
