//! The `Prop` type wraps a function that is called to compute the value
//! of an attribute.
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use feint_code_gen::*;

use super::base::{ObjectRef, ObjectTrait, TypeRef};

use super::ns::Namespace;

// Prop ---------------------------------------------------------

pub struct Prop {
    class: TypeRef,

    ns: Namespace,
    getter: ObjectRef,
}

standard_object_impls!(Prop);

impl Prop {
    pub fn new(class: TypeRef, getter: ObjectRef) -> Self {
        Self { class, ns: Namespace::default(), getter }
    }

    pub fn getter(&self) -> ObjectRef {
        self.getter.clone()
    }
}

impl ObjectTrait for Prop {
    object_trait_header!();
}

// Display -------------------------------------------------------------

impl fmt::Display for Prop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<prop: {}>", self.getter.read().unwrap())
    }
}

impl fmt::Debug for Prop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
