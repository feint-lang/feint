use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use feint_code_gen::*;

use super::base::{ObjectRef, ObjectTrait, TypeRef};

use super::ns::Namespace;

// Nil ----------------------------------------------------------

pub struct Nil {
    class: TypeRef,
    ns: Namespace,
}

standard_object_impls!(Nil);

impl Nil {
    #[allow(clippy::new_without_default)]
    pub fn new(class: TypeRef) -> Self {
        Self { class, ns: Namespace::default() }
    }
}

impl ObjectTrait for Nil {
    object_trait_header!();

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
