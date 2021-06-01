use std::collections::HashMap;
use std::rc::Rc;

use num_bigint::BigInt;
use num_traits::Num;

use super::class::{Type, TypeRef};
use super::object::{Object, ObjectExt, ObjectRef};

pub struct Builtins {
    types: HashMap<&'static str, TypeRef>,
    pub nil_obj: Rc<super::nil::Nil>,
    pub true_obj: Rc<super::bool::Bool>,
    pub false_obj: Rc<super::bool::Bool>,
}

impl Builtins {
    pub fn new() -> Self {
        let mut types = HashMap::new();

        // Singleton types
        let nil_type = Self::create_type("Nil");
        let bool_type = Self::create_type("Bool");

        // Singletons
        let nil_obj = Rc::new(super::nil::Nil::new(nil_type.clone()));
        let true_obj = Rc::new(super::bool::Bool::new(bool_type.clone(), true));
        let false_obj = Rc::new(super::bool::Bool::new(bool_type.clone(), false));

        // All the builtin types
        types.insert("Nil", nil_type);
        types.insert("Bool", bool_type);
        types.insert("Float", Self::create_type("Float"));
        types.insert("Int", Self::create_type("Int"));
        types.insert("String", Self::create_type("String"));

        Self { types, nil_obj, true_obj, false_obj }
    }

    fn create_type(name: &'static str) -> TypeRef {
        Rc::new(Type::new("builtins", name))
    }

    /// Get builtin type by name. Panic if a type doesn't exist with the
    /// specified name.
    fn get_type(&self, name: &str) -> &TypeRef {
        let message = format!("Unknown builtin type: {}", name);
        let class = self.types.get(name).expect(message.as_str());
        class
    }

    // Builtin type constructors

    pub fn new_float<F: Into<f64>>(&self, value: F) -> ObjectRef {
        let class = self.get_type("Float").clone();
        let value = value.into();
        Rc::new(super::float::Float::new(class, value))
    }

    pub fn new_float_from_string<S: Into<String>>(&self, value: S) -> ObjectRef {
        let value = value.into();
        let value = value.parse::<f64>().unwrap();
        self.new_float(value)
    }

    pub fn new_int<I: Into<BigInt>>(&self, value: I) -> ObjectRef {
        let class = self.get_type("Int").clone();
        let value = value.into();
        Rc::new(super::int::Int::new(class, value))
    }

    pub fn new_int_from_string<S: Into<String>>(&self, value: S) -> ObjectRef {
        let value = value.into();
        let value = BigInt::from_str_radix(value.as_ref(), 10).unwrap();
        self.new_int(value)
    }

    pub fn new_string<S: Into<String>>(&self, value: S, format: bool) -> ObjectRef {
        let class = self.get_type("String").clone();
        let value = value.into();
        Rc::new(super::string::String::new(class, value, format))
    }
}
