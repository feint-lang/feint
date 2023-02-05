//! Builtin `Nil` type.
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use feint_code_gen::*;

use super::base::{ObjectTrait, TypeRef};
use super::class::Type;
use super::ns::Namespace;

std_type!(NIL_TYPE, NilType);

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
