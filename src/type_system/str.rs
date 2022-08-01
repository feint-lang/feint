use std::any::Any;
use std::fmt;
use std::sync::Arc;

use once_cell::sync::Lazy;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::builtins::BUILTINS;
use super::class::TYPE_TYPE;
use super::ns::Namespace;

use super::create;

// Str Type ------------------------------------------------------------

pub static STR_TYPE: Lazy<Arc<StrType>> = Lazy::new(|| Arc::new(StrType::new()));

pub struct StrType {
    namespace: Arc<Namespace>,
}

impl StrType {
    pub fn new() -> Self {
        let mut ns = Namespace::new();
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

    fn namespace(&self) -> ObjectRef {
        self.namespace.clone()
    }
}

impl ObjectTrait for StrType {
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

    fn metaclass(&self) -> TypeRef {
        STR_TYPE.clone()
    }

    fn class(&self) -> ObjectRef {
        STR_TYPE.clone()
    }

    fn namespace(&self) -> ObjectRef {
        self.namespace.clone()
    }
}

// Display -------------------------------------------------------------

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
