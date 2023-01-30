use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::new;
use feint_code_gen::*;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Nil Type ------------------------------------------------------------

type_and_impls!(NilType, Nil);

pub static NIL_TYPE: Lazy<obj_ref_t!(NilType)> = Lazy::new(|| obj_ref!(NilType::new()));

// Nil Object ----------------------------------------------------------

pub struct Nil {
    ns: Namespace,
}

standard_object_impls!(Nil);

impl Nil {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { ns: Namespace::default() }
    }
}

impl ObjectTrait for Nil {
    object_trait_header!(NIL_TYPE);

    fn bool_val(&self) -> Option<bool> {
        Some(false)
    }
}

// Display -------------------------------------------------------------

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
