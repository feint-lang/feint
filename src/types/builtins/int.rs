use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use num_bigint::BigInt;

use super::super::class::Type;
use super::super::object::Object;
use super::float::Float;
use super::BUILTIN_TYPES;

/// Built in integer type
#[derive(Debug, PartialEq)]
pub struct Int {
    value: BigInt,
}

impl Int {
    pub fn value(&self) -> &BigInt {
        &self.value
    }

    pub fn eq_float(&self, float: &Float) -> bool {
        float.eq_int(self)
    }
}

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<BigInt> for Int {
    fn from(value: BigInt) -> Self {
        Int { value }
    }
}

impl From<i32> for Int {
    fn from(value: i32) -> Self {
        Int { value: BigInt::from(value) }
    }
}

impl Object for Int {
    fn class(&self) -> Arc<Type> {
        BUILTIN_TYPES.get("Int").unwrap().clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
