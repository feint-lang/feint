use std::any::Any;
use std::fmt;
use std::sync::Arc;

use once_cell::sync::Lazy;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::builtins::BUILTINS;
use super::class::TYPE_TYPE;
use super::ns::Namespace;

use super::create;

// Bool Type -----------------------------------------------------------

pub static BOOL_TYPE: Lazy<Arc<BoolType>> = Lazy::new(|| Arc::new(BoolType::new()));

pub struct BoolType {
    namespace: Arc<Namespace>,
}

unsafe impl Send for BoolType {}
unsafe impl Sync for BoolType {}

impl BoolType {
    pub fn new() -> Self {
        let mut ns = Namespace::new();
        ns.add_obj("$name", create::new_str("Bool"));
        ns.add_obj("$full_name", create::new_str("builtins.Bool"));
        Self { namespace: Arc::new(ns) }
    }
}

impl TypeTrait for BoolType {
    fn name(&self) -> &str {
        "Bool"
    }

    fn full_name(&self) -> &str {
        "builtins.Bool"
    }

    fn namespace(&self) -> ObjectRef {
        self.namespace.clone()
    }
}

impl ObjectTrait for BoolType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn metaclass(&self) -> TypeRef {
        TYPE_TYPE.clone()
    }

    fn class(&self) -> ObjectRef {
        TYPE_TYPE.clone()
    }

    fn namespace(&self) -> ObjectRef {
        self.namespace.clone()
    }
}

// Bool Object ---------------------------------------------------------

pub struct Bool {
    namespace: Arc<Namespace>,
    value: bool,
}

unsafe impl Send for Bool {}
unsafe impl Sync for Bool {}

impl Bool {
    pub fn new(value: bool) -> Self {
        Self { namespace: Arc::new(Namespace::new()), value }
    }

    pub fn value(&self) -> &bool {
        &self.value
    }
}

impl ObjectTrait for Bool {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn metaclass(&self) -> TypeRef {
        BOOL_TYPE.clone()
    }

    fn class(&self) -> ObjectRef {
        BOOL_TYPE.clone()
    }

    fn namespace(&self) -> ObjectRef {
        self.namespace.clone()
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Debug for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
