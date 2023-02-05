//! Builtin `Always` type.
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use feint_code_gen::*;

use super::base::{ObjectTrait, TypeRef};
use super::class::Type;
use super::ns::Namespace;

std_type!(ALWAYS_TYPE, AlwaysType);

pub struct Always {
    ns: Namespace,
}

standard_object_impls!(Always);

impl Always {
    pub fn new() -> Self {
        Self { ns: Namespace::default() }
    }
}

impl ObjectTrait for Always {
    object_trait_header!(ALWAYS_TYPE);

    fn is_equal(&self, _rhs: &dyn ObjectTrait) -> bool {
        true
    }
}

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
