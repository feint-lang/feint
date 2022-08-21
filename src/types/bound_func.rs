use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Bound Function Type -------------------------------------------------

gen::type_and_impls!(BoundFuncType, BoundFunc);

pub static BOUND_FUNC_TYPE: Lazy<Arc<RwLock<BoundFuncType>>> =
    Lazy::new(|| Arc::new(RwLock::new(BoundFuncType::new())));

// BoundFunc Object ----------------------------------------------------------

pub struct BoundFunc {
    ns: Namespace,
    pub func: ObjectRef,
    pub this: ObjectRef,
}

gen::standard_object_impls!(BoundFunc);

impl BoundFunc {
    pub fn new(func: ObjectRef, this: ObjectRef) -> Self {
        Self { ns: Namespace::new(), func, this }
    }
}

impl ObjectTrait for BoundFunc {
    gen::object_trait_header!(BOUND_FUNC_TYPE, BoundFunc);
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
