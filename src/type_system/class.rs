//! "Class" and "type" are used interchangeably and mean exactly the
//! same thing. Lower case "class" is used instead of "type" because the
//! latter is a Rust keyword.
use std::any::Any;
use std::fmt;
use std::sync::Arc;

use once_cell::sync::Lazy;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::builtins::BUILTINS;
use super::create;
use super::ns::Namespace;

// Type Type -----------------------------------------------------------

pub static TYPE_TYPE: Lazy<Arc<TypeType>> = Lazy::new(|| Arc::new(TypeType::new()));

pub struct TypeType {
    namespace: Arc<Namespace>,
}

impl TypeType {
    pub fn new() -> Self {
        let mut ns = Namespace::new();
        ns.add_obj("$name", create::new_str("Type"));
        ns.add_obj("$full_name", create::new_str("builtins.Type"));
        Self { namespace: Arc::new(ns) }
    }
}

unsafe impl Send for TypeType {}
unsafe impl Sync for TypeType {}

impl TypeTrait for TypeType {
    fn name(&self) -> &str {
        "Type"
    }

    fn full_name(&self) -> &str {
        "builtins.Type"
    }

    fn namespace(&self) -> ObjectRef {
        self.namespace.clone()
    }
}

impl ObjectTrait for TypeType {
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

// Type Object ---------------------------------------------------------

pub struct Type {
    namespace: Arc<Namespace>,
}

unsafe impl Send for Type {}
unsafe impl Sync for Type {}

impl Type {
    pub fn new() -> Self {
        let ns = Namespace::new();
        Self { namespace: Arc::new(ns) }
    }
}

impl ObjectTrait for Type {
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

// Display -------------------------------------------------------------

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.class(), self.id())
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
