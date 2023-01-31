use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use feint_code_gen::*;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::func_trait::FuncTrait;
use super::ns::Namespace;
use super::{new, Params};

// Closure Type --------------------------------------------------------

type_and_impls!(ClosureType, Closure);

pub static CLOSURE_TYPE: Lazy<obj_ref_t!(ClosureType)> =
    Lazy::new(|| obj_ref!(ClosureType::new()));

// Closure Object ------------------------------------------------------

pub struct Closure {
    ns: Namespace,
    module_name: String,
    name: String,
    params: Params,
    func: ObjectRef,
    /// `Map<Str, Cell>` of captured vars
    captured: ObjectRef,
}

standard_object_impls!(Closure);

impl Closure {
    pub fn new(func_ref: ObjectRef, captured: ObjectRef) -> Self {
        let func = func_ref.read().unwrap();
        let func = func.down_to_func().unwrap();
        Self {
            ns: Namespace::with_entries(&[("$doc", func.get_doc())]),
            module_name: func.module_name().to_owned(),
            name: func.name().to_owned(),
            params: func.params().clone(),
            func: func_ref.clone(),
            captured,
        }
    }

    pub fn func(&self) -> ObjectRef {
        self.func.clone()
    }

    pub fn captured(&self) -> ObjectRef {
        self.captured.clone()
    }

    /// Get cell for `name`, if `name` was captured by this closure.
    pub fn get_captured(&self, name: &str) -> Option<ObjectRef> {
        let captured = self.captured.read().unwrap();
        let captured = captured.down_to_map().unwrap();
        captured.get(name)
    }
}

impl FuncTrait for Closure {
    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn module_name(&self) -> &String {
        &self.module_name
    }

    fn module(&self) -> ObjectRef {
        self.func().read().unwrap().module()
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn params(&self) -> &Params {
        &self.params
    }
}

impl ObjectTrait for Closure {
    object_trait_header!(CLOSURE_TYPE);
}

// Display -------------------------------------------------------------

impl fmt::Display for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[closure] {}", self.func.read().unwrap())
    }
}

impl fmt::Debug for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[closure] {:?}", &*self.func.read().unwrap())
    }
}
