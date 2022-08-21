use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Bound Function Type -------------------------------------------------

pub static BOUND_FUNC_TYPE: Lazy<Arc<RwLock<BoundFuncType>>> =
    Lazy::new(|| Arc::new(RwLock::new(BoundFuncType::new())));

pub struct BoundFuncType {
    ns: Namespace,
}

unsafe impl Send for BoundFuncType {}
unsafe impl Sync for BoundFuncType {}

impl BoundFuncType {
    pub fn new() -> Self {
        Self {
            ns: Namespace::with_entries(&[
                // Class Attributes
                ("$name", new::str("BoundFunc")),
                ("$full_name", new::str("builtins.BoundFunc")),
            ]),
        }
    }
}

impl TypeTrait for BoundFuncType {
    fn name(&self) -> &str {
        "BoundFunc"
    }

    fn full_name(&self) -> &str {
        "builtins.BoundFunc"
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }
}

impl ObjectTrait for BoundFuncType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        TYPE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        TYPE_TYPE.clone()
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
    }
}

// BoundFunc Object ----------------------------------------------------------

pub struct BoundFunc {
    ns: Namespace,
    pub func: ObjectRef,
    pub this: ObjectRef,
}

unsafe impl Send for BoundFunc {}
unsafe impl Sync for BoundFunc {}

impl BoundFunc {
    pub fn new(func: ObjectRef, this: ObjectRef) -> Self {
        Self { ns: Namespace::new(), func, this }
    }
}

impl ObjectTrait for BoundFunc {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        BOUND_FUNC_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        BOUND_FUNC_TYPE.clone()
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
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
