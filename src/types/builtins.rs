use std::collections::HashMap;
use std::rc::Rc;

use num_bigint::BigInt;

use crate::ast::ExpressionKind::Function;

use super::object::{Attribute, Fundamental, ObjectTrait};
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
    pub none_singleton: Rc<Fundamental>,
    pub true_singleton: Rc<Fundamental>,
    pub false_singleton: Rc<Fundamental>,
    pub float: Rc<Type>,
    pub int: Rc<Type>,
}

impl Builtins {
    pub fn new() -> Self {
        let none_type = Rc::new(Type::new("builtins", "None", None));
        let bool_type = Rc::new(Type::new("builtins", "Bool", Some(vec!["value"])));
        Self {
            none_singleton: Rc::new(Fundamental::None(none_type.clone())),
            true_singleton: Rc::new(Fundamental::Bool(bool_type.clone(), true)),
            false_singleton: Rc::new(Fundamental::Bool(bool_type.clone(), false)),
            float: Rc::new(Type::new("builtins", "Float", Some(vec!["value"]))),
            int: Rc::new(Type::new("builtins", "Int", Some(vec!["value"]))),
        }
    }

    pub fn new_float(&self, value: f64) -> Box<dyn ObjectTrait> {
        Box::new(Fundamental::Float(self.float.clone(), value))
    }

    pub fn new_int(&self, value: BigInt) -> Box<dyn ObjectTrait> {
        Box::new(Fundamental::Int(self.int.clone(), value))
    }
}
