//! The `Prop` type wraps a function that is called to compute the value
//! of an attribute.
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use feint_code_gen::*;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::new;
use super::ns::Namespace;

// Prop Type -----------------------------------------------------------

type_and_impls!(PropType, Prop);

pub static PROP_TYPE: Lazy<obj_ref_t!(PropType)> =
    Lazy::new(|| obj_ref!(PropType::new()));

// Prop Object ---------------------------------------------------------

pub struct Prop {
    ns: Namespace,
    getter: ObjectRef,
}

standard_object_impls!(Prop);

impl Prop {
    pub fn new(getter: ObjectRef) -> Self {
        Self { ns: Namespace::default(), getter }
    }

    pub fn getter(&self) -> ObjectRef {
        self.getter.clone()
    }
}

impl ObjectTrait for Prop {
    object_trait_header!(PROP_TYPE);
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
