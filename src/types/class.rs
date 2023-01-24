//! "Class" and "type" are used interchangeably and mean exactly the
//! same thing. Lower case "class" is used instead of "type" because the
//! latter is a Rust keyword.
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::ns::Namespace;

// Type Type -----------------------------------------------------------

gen::type_and_impls!(TypeType, Type);

pub static TYPE_TYPE: Lazy<gen::obj_ref_t!(TypeType)> =
    Lazy::new(|| gen::obj_ref!(TypeType::new()));

// Type Object ---------------------------------------------------------

pub struct Type {
    ns: Namespace,
}

gen::standard_object_impls!(Type);

impl Type {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { ns: Namespace::default() }
    }
}

impl ObjectTrait for Type {
    gen::object_trait_header!(TYPE_TYPE);
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
