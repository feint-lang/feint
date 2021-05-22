use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use num_bigint::BigInt;

use super::super::class::Type;
use super::super::object::Object;
use super::int::Int;
use super::BUILTIN_TYPES;

/// Built in 64-bit float type
#[derive(Debug, PartialEq)]
pub struct Float {
    value: f64,
}

impl Float {
    pub fn eq_int(&self, int: &Int) -> bool {
        self.value.fract() == 0.0 && BigInt::from(self.value as i128) == *int.value()
    }
}

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<f64> for Float {
    fn from(value: f64) -> Self {
        Float { value }
    }
}

impl Object for Float {
    fn class(&self) -> Arc<Type> {
        BUILTIN_TYPES.get("Float").unwrap().clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
