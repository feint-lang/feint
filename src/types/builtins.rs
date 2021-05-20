use std::collections::HashMap;
use std::rc::Rc;

use num_bigint::BigInt;

use super::Type;
use crate::types::object::Builtin;

/// Built in boolean type
pub struct Bool {
    value: bool,
}

/// Built in 64-bit float type
pub struct Float {
    value: f64,
}

impl From<f64> for Float {
    fn from(value: f64) -> Self {
        Float { value }
    }
}

/// Built in integer type
pub struct Int {
    value: BigInt,
}

impl From<BigInt> for Int {
    fn from(value: BigInt) -> Self {
        Int { value }
    }
}

pub struct Builtins {
    pub none: Rc<Type>,
    pub bool: Rc<Type>,
    pub float: Rc<Type>,
    pub int: Rc<Type>,
}

impl Builtins {
    pub fn new() -> Self {
        Self {
            none: Rc::new(Type::new("builtins", "None", vec![])),
            bool: Rc::new(Type::new("builtins", "Bool", vec!["value"])),
            float: Rc::new(Type::new("builtins", "Float", vec!["value"])),
            int: Rc::new(Type::new("builtins", "Int", vec!["value"])),
        }
    }

    pub fn new_float(&self, value: f64) -> Builtin {
        Builtin::Float(self.float.clone(), value)
    }

    pub fn new_int(&self, value: BigInt) -> Builtin {
        Builtin::Int(self.int.clone(), value)
    }
}
