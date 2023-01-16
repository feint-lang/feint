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

pub static CLOSURE_TYPE: Lazy<gen::obj_ref_t!(ClosureType)> =
    Lazy::new(|| gen::obj_ref!(ClosureType::new()));

// Closure Object ------------------------------------------------------

pub struct Closure {
    ns: Namespace,
    name: String,
    params: Params,
    func: ObjectRef,
    captured: IndexMap<String, ObjectRef>,
}

gen::standard_object_impls!(Closure);

impl Closure {
    pub fn new(func_ref: ObjectRef, captured: IndexMap<String, ObjectRef>) -> Self {
        let func = func_ref.read().unwrap();
        let func = func.down_to_func().unwrap();
        Self {
            ns: Namespace::with_entries(&[("$doc", func.get_doc())]),
            name: func.name().to_owned(),
            params: func.params().clone(),
            func: func_ref.clone(),
            captured,
        }
    }

    pub fn func(&self) -> ObjectRef {
        self.func.clone()
    }

    pub fn captured(&self) -> &IndexMap<String, ObjectRef> {
        &self.captured
    }
}

impl FuncTrait for Closure {
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
