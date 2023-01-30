use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use feint_code_gen::*;

use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Bool Type -----------------------------------------------------------

type_and_impls!(BoolType, Bool);

pub static BOOL_TYPE: Lazy<obj_ref_t!(BoolType)> =
    Lazy::new(|| obj_ref!(BoolType::new()));

// Bool Object ---------------------------------------------------------

pub struct Bool {
    ns: Namespace,
    value: bool,
}

standard_object_impls!(Bool);

impl Bool {
    pub fn new(value: bool) -> Self {
        Self { ns: Namespace::default(), value }
    }

    pub fn value(&self) -> &bool {
        &self.value
    }
}

impl ObjectTrait for Bool {
    object_trait_header!(BOOL_TYPE);

    // Unary operations -----------------------------------------------

    fn bool_val(&self) -> Option<bool> {
        Some(*self.value())
    }

    // Binary operations -----------------------------------------------

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if self.is(rhs) || rhs.is_always() {
            true
        } else if let Some(rhs) = rhs.down_to_bool() {
            self.value() == rhs.value()
        } else {
            false
        }
    }

    fn and(&self, rhs: &dyn ObjectTrait) -> Option<bool> {
        if let Some(rhs) = rhs.down_to_bool() {
            Some(*self.value() && *rhs.value())
        } else {
            None
        }
    }

    fn or(&self, rhs: &dyn ObjectTrait) -> Option<bool> {
        if let Some(rhs) = rhs.down_to_bool() {
            Some(*self.value() || *rhs.value())
        } else {
            None
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Debug for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
