use std::collections::HashMap;
use std::rc::Rc;

use num_bigint::BigInt;

use crate::ast::ExpressionKind::Function;

use super::object::{FundamentalObject, Object};
use super::types::Type;

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
    pub none_singleton: Rc<Object>,
    pub true_singleton: Rc<Object>,
    pub false_singleton: Rc<Object>,
    pub float: Rc<Type>,
    pub int: Rc<Type>,
}

impl Builtins {
    pub fn new() -> Self {
        let none_type = Rc::new(Type::new("builtins", "None"));
        let bool_type = Rc::new(Type::new("builtins", "Bool"));
        Self {
            none_singleton: Rc::new(FundamentalObject::None(none_type.clone())),
            true_singleton: Rc::new(FundamentalObject::Bool(bool_type.clone(), true)),
            false_singleton: Rc::new(FundamentalObject::Bool(bool_type.clone(), false)),

            float: Rc::new(Type::new("builtins", "Float")),
            int: Rc::new(Type::new("builtins", "Int")),
        }
    }

    pub fn new_float(&self, value: f64) -> Rc<Object> {
        Rc::new(FundamentalObject::Float(self.float.clone(), value))
    }

    pub fn new_int(&self, value: BigInt) -> Rc<Object> {
        Rc::new(FundamentalObject::Int(self.int.clone(), value))
    }
}
