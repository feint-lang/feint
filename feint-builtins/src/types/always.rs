use std::any::Any;
use std::fmt;

use feint_code_gen::*;

use super::base::{ObjectRef, ObjectTrait, TypeRef};
use super::ns::Namespace;

// Always --------------------------------------------------------------

pub struct Always {
    class: TypeRef,
    ns: Namespace,
}

standard_object_impls!(Always);

impl Always {
    pub fn new(class: TypeRef) -> Self {
        Self { class, ns: Namespace::default() }
    }
}

impl ObjectTrait for Always {
    object_trait_header!();

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
