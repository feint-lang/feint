use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// AlwaysType Type -----------------------------------------------------

gen::type_and_impls!(AlwaysType, Always);

pub static ALWAYS_TYPE: Lazy<new::obj_ref_t!(AlwaysType)> =
    Lazy::new(|| new::obj_ref!(AlwaysType::new()));

// Always Object -------------------------------------------------------

pub struct Always {
    ns: Namespace,
}

gen::standard_object_impls!(Always);

impl Always {
    pub fn new() -> Self {
        Self { ns: Namespace::new() }
    }
}

impl ObjectTrait for Always {
    gen::object_trait_header!(ALWAYS_TYPE);

    fn is_equal(&self, _rhs: &dyn ObjectTrait) -> bool {
        true
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Always {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@")
    }
}

impl fmt::Debug for Always {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}