use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::create;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Bound Function Type -------------------------------------------------

pub static BOUND_FUNC_TYPE: Lazy<Arc<RwLock<BoundFuncType>>> =
    Lazy::new(|| Arc::new(RwLock::new(BoundFuncType::new())));

pub struct BoundFuncType {
    namespace: Namespace,
}

unsafe impl Send for BoundFuncType {}
unsafe impl Sync for BoundFuncType {}

impl BoundFuncType {
    pub fn new() -> Self {
        let mut ns = Namespace::new();
        ns.add_obj("$name", create::new_str("BoundFunc"));
        ns.add_obj("$full_name", create::new_str("builtins.BoundFunc"));
        Self { namespace: ns }
    }
}

impl TypeTrait for BoundFuncType {
    fn name(&self) -> &str {
        "BoundFunc"
    }

    fn full_name(&self) -> &str {
        "builtins.BoundFunc"
    }
}

impl ObjectTrait for BoundFuncType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        TYPE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        TYPE_TYPE.clone()
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

// BoundFunc Object ----------------------------------------------------------

pub struct BoundFunc {
    namespace: Namespace,
    pub func: ObjectRef,
    pub this: ObjectRef,
}

unsafe impl Send for BoundFunc {}
unsafe impl Sync for BoundFunc {}

impl BoundFunc {
    pub fn new(func: ObjectRef, this: ObjectRef) -> Self {
        Self { namespace: Namespace::new(), func, this }
    }
}

impl ObjectTrait for BoundFunc {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        BOUND_FUNC_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        BOUND_FUNC_TYPE.clone()
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
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
