//! "Class" and "type" are used interchangeably and mean exactly the
//! same thing. Lower case "class" is used instead of "type" because the
//! latter is a Rust keyword.
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::ns::Namespace;

// Type Type -----------------------------------------------------------

pub static TYPE_TYPE: Lazy<new::obj_ref_t!(TypeType)> =
    Lazy::new(|| new::obj_ref!(TypeType::new()));

pub struct TypeType {
    ns: Namespace,
}

impl TypeType {
    pub fn new() -> Self {
        Self {
            ns: Namespace::with_entries(&[
                // Class Attributes
                ("$name", new::str("Type")),
                ("$full_name", new::str("builtins.Type")),
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

    fn ns(&self) -> &Namespace {
        &self.ns
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

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
    }
}

// Type Object ---------------------------------------------------------

pub struct Type {
    ns: Namespace,
}

unsafe impl Send for Type {}
unsafe impl Sync for Type {}

impl Type {
    pub fn new() -> Self {
        let ns = Namespace::new();
        Self { ns }
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

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
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
