use std::any::Any;
use std::fmt;
use std::sync::Arc;

use once_cell::sync::Lazy;

use super::base::{ObjectRef, ObjectTrait, TypeTrait};
use super::class::TYPE;
use super::create;
use super::ns::Namespace;

// Bool Type -----------------------------------------------------------

static BOOL_TYPE: Lazy<Arc<BoolType>> = Lazy::new(|| Arc::new(BoolType::new()));

pub struct BoolType {
    namespace: Arc<Namespace>,
}

unsafe impl Send for BoolType {}
unsafe impl Sync for BoolType {}

impl BoolType {
    pub fn new() -> Self {
        let mut ns = Namespace::new();
        ns.add_obj("$module", create::new_str("builtins"));
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
}

impl ObjectTrait for BoolType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> ObjectRef {
        TYPE.clone()
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

    fn class(&self) -> ObjectRef {
        BOOL_TYPE.clone()
    }

    fn namespace(&self) -> ObjectRef {
        self.namespace.clone()
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for BoolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}.{}>", self.module(), self.name())
    }
}

impl fmt::Debug for BoolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

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
