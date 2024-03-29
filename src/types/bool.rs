use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::{RuntimeBoolResult, RuntimeErr};

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Bool Type -----------------------------------------------------------

gen::type_and_impls!(BoolType, Bool);

pub static BOOL_TYPE: Lazy<gen::obj_ref_t!(BoolType)> =
    Lazy::new(|| gen::obj_ref!(BoolType::new()));

// Bool Object ---------------------------------------------------------

pub struct Bool {
    ns: Namespace,
    value: bool,
}

gen::standard_object_impls!(Bool);

impl Bool {
    pub fn new(value: bool) -> Self {
        Self { ns: Namespace::default(), value }
    }

    pub fn value(&self) -> &bool {
        &self.value
    }
}

impl ObjectTrait for Bool {
    gen::object_trait_header!(BOOL_TYPE);

    // Unary operations -----------------------------------------------

    fn bool_val(&self) -> RuntimeBoolResult {
        Ok(*self.value())
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

    fn and(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_bool() {
            Ok(*self.value() && *rhs.value())
        } else {
            Err(RuntimeErr::type_err(format!(
                "{} && {} not implemented",
                self.class().read().unwrap(),
                rhs.class().read().unwrap(),
            )))
        }
    }

    fn or(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_bool() {
            Ok(*self.value() || *rhs.value())
        } else {
            Err(RuntimeErr::type_err(format!(
                "{} || {} not implemented",
                self.class().read().unwrap(),
                rhs.class().read().unwrap(),
            )))
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
