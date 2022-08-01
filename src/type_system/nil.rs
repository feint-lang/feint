use std::any::Any;
use std::fmt;
use std::sync::Arc;

use once_cell::sync::Lazy;

use super::base::{ObjectRef, ObjectTrait, TypeTrait};
use super::class::TYPE;
use super::create;
use super::ns::Namespace;

// Nil Type ------------------------------------------------------------

static NIL_TYPE: Lazy<Arc<NilType>> = Lazy::new(|| Arc::new(NilType::new()));

pub struct NilType {
    namespace: Arc<Namespace>,
}

unsafe impl Send for NilType {}
unsafe impl Sync for NilType {}

impl NilType {
    pub fn new() -> Self {
        let mut ns = Namespace::new();
        ns.add_obj("$module", create::new_str("builtins"));
        ns.add_obj("$name", create::new_str("Nil"));
        ns.add_obj("$full_name", create::new_str("builtins.Nil"));
        Self { namespace: Arc::new(ns) }
    }
}

impl TypeTrait for NilType {
    fn name(&self) -> &str {
        "Nil"
    }

    fn full_name(&self) -> &str {
        "builtins.Nil"
    }
}

impl ObjectTrait for NilType {
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

// Nil Object ----------------------------------------------------------

pub struct Nil {
    namespace: Arc<Namespace>,
}

unsafe impl Send for Nil {}
unsafe impl Sync for Nil {}

impl Nil {
    pub fn new() -> Self {
        Self { namespace: Arc::new(Namespace::new()) }
    }
}

impl ObjectTrait for Nil {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> ObjectRef {
        NIL_TYPE.clone()
    }

    fn namespace(&self) -> ObjectRef {
        self.namespace.clone()
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for NilType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}.{}>", self.module(), self.name())
    }
}

impl fmt::Debug for NilType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Nil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "nil")
    }
}

impl fmt::Debug for Nil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
