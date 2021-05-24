use std::collections::HashMap;
use std::rc::Rc;

use num_bigint::BigInt;

use super::super::{Object, Type};

pub struct Builtins {
    types: HashMap<&'static str, Rc<Type>>,
    pub nil_obj: Rc<super::Nil>,
    pub true_obj: Rc<super::Bool>,
    pub false_obj: Rc<super::Bool>,
}

impl Builtins {
    pub fn new() -> Self {
        let mut types = HashMap::new();

        // Singleton types
        let nil_type = Self::create_type("Nil");
        let bool_type = Self::create_type("Bool");

        // Singletons
        let nil_obj = Rc::new(super::Nil::new(nil_type.clone()));
        let true_obj = Rc::new(super::Bool::new(bool_type.clone(), true));
        let false_obj = Rc::new(super::Bool::new(bool_type.clone(), false));

        // All the builtin types
        types.insert("Nil", nil_type);
        types.insert("Bool", bool_type);
        types.insert("Float", Self::create_type("Float"));
        types.insert("Int", Self::create_type("Int"));

        Self { types, nil_obj, true_obj, false_obj }
    }

    fn create_type(name: &'static str) -> Rc<Type> {
        Rc::new(Type::new("builtins", name))
    }

    /// Get builtin type by name. Panic if a type doesn't exist with the
    /// specified name.
    fn get_type(&self, name: &str) -> &Rc<Type> {
        let message = format!("Unknown builtin type: {}", name);
        let class = self.types.get(name).expect(message.as_str());
        class
    }

    // Builtin type constructors

    pub fn new_float<F: Into<f64>>(&self, value: F) -> Rc<dyn Object> {
        let class = self.get_type("Float").clone();
        let value = value.into();
        Rc::new(super::Float::new(class, value))
    }

    pub fn new_int<I: Into<BigInt>>(&self, value: I) -> Rc<dyn Object> {
        let class = self.get_type("Float").clone();
        let value = value.into();
        Rc::new(super::Int::new(class, value))
    }
}
