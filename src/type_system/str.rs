use std::any::Any;
use std::fmt;
use std::sync::Arc;

use once_cell::sync::Lazy;

use super::base::{ObjectRef, ObjectTrait, TypeTrait};
use super::class::TYPE;
use super::create;
use super::ns::Namespace;

// Str Type ------------------------------------------------------------

static STR_TYPE: Lazy<Arc<StrType>> = Lazy::new(|| Arc::new(StrType::new()));

pub struct StrType {
    namespace: Arc<Namespace>,
}

impl StrType {
    pub fn new() -> Self {
        let mut ns = Namespace::new();
        ns.add_obj("$module", create::new_str("builtins"));
        ns.add_obj("$name", create::new_str("Str"));
        ns.add_obj("$full_name", create::new_str("builtins.Str"));
        Self { namespace: Arc::new(ns) }
    }
}

unsafe impl Send for StrType {}
unsafe impl Sync for StrType {}

impl TypeTrait for StrType {
    fn name(&self) -> &str {
        "Str"
    }

    fn full_name(&self) -> &str {
        "builtins.Str"
    }
}

impl ObjectTrait for StrType {
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

// Str Object ----------------------------------------------------------

pub struct Str {
    namespace: Arc<Namespace>,
    value: String,
}

impl Str {
    pub fn new<S: Into<String>>(value: S) -> Self {
        Self { namespace: Arc::new(Namespace::new()), value: value.into() }
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

impl ObjectTrait for Str {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> ObjectRef {
        STR_TYPE.clone()
    }

    fn namespace(&self) -> ObjectRef {
        self.namespace.clone()
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for StrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}.{}>", self.module(), self.name())
    }
}

impl fmt::Debug for StrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Debug for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}
