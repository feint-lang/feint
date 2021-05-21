use std::any::Any;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::sync::Arc;

use lazy_static::lazy_static;

use num_bigint::BigInt;

use super::object::Object;
use super::types::Type;

lazy_static! {
    pub static ref BUILTIN_TYPES: HashMap<&'static str, Arc<Type>> = {
        let mut builtins = HashMap::new();
        let mut insert = |n| builtins.insert(n, Arc::new(Type::new("builtins", n)));
        insert("None");
        insert("Bool");
        insert("Float");
        insert("Int");
        builtins
    };
}

/// Built in boolean type
#[derive(Debug, PartialEq)]
pub struct Bool {
    value: bool,
}

impl Display for Bool {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<bool> for Bool {
    fn from(value: bool) -> Self {
        Bool { value }
    }
}

impl Object for Bool {
    fn class(&self) -> Arc<Type> {
        BUILTIN_TYPES.get("Bool").unwrap().clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Built in 64-bit float type
#[derive(Debug, PartialEq)]
pub struct Float {
    value: f64,
}

impl Float {
    pub fn eq_int(&self, int: &Int) -> bool {
        self.value.fract() == 0.0 && BigInt::from(self.value as i128) == int.value
    }
}

impl Display for Float {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

/// Built in integer type
#[derive(Debug, PartialEq)]
pub struct Int {
    value: BigInt,
}

impl Int {
    pub fn eq_float(&self, float: &Float) -> bool {
        float.eq_int(self)
    }
}

impl Display for Int {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
