//! "Class" and "type" are used interchangeably and mean exactly the
//! same thing. Lower case "class" is used instead of "type" because the
//! latter is a Rust keyword.
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::create;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::ns::Namespace;

// Type Type -----------------------------------------------------------

pub static TYPE_TYPE: Lazy<Arc<RwLock<TypeType>>> =
    Lazy::new(|| Arc::new(RwLock::new(TypeType::new())));

pub struct TypeType {
    namespace: Namespace,
}

impl TypeType {
    pub fn new() -> Self {
        Self {
            namespace: Namespace::with_entries(&[
                // Class Attributes
                ("$name", create::new_str("Type")),
                ("$full_name", create::new_str("builtins.Type")),
            ]),
        }
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

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

impl ObjectTrait for TypeType {
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

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

// Type Object ---------------------------------------------------------

pub struct Type {
    namespace: Namespace,
}

unsafe impl Send for Type {}
unsafe impl Sync for Type {}

impl Type {
    pub fn new() -> Self {
        let ns = Namespace::new();
        Self { namespace: ns }
    }
}

impl ObjectTrait for Type {
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

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.type_obj().read().unwrap(), self.id())
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
