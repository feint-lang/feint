use std::collections::HashMap;
use std::fmt;

use num_bigint::BigInt;

use super::{AttributeValue, Object, ObjectTrait, Primitive, BUILTIN_TYPES};

pub struct Type {
    pub module: String,
    pub name: String,
    pub slots: Vec<String>,
}

impl Type {
    /// Create a new type.
    pub fn new(module: &str, name: &str, slots: Vec<&str>) -> Self {
        let module = module.to_owned();
        let name = name.to_owned();
        let slots = slots.iter().map(|s| (*s).to_owned()).collect();
        Self { module, name, slots }
    }

    // Unique ID for type.
    pub fn id(&self) -> *const Self {
        self as *const Self
    }

    /// Is this type the other type?
    pub fn is(&self, other: &Self) -> bool {
        self.id() == other.id()
    }

    pub fn equal(&self, other: &Self) -> bool {
        other.is(self)
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}({})", self.module, self.name, self.slots.join(", "))
    }

    pub fn instance<'b>(&'b self, values: Vec<AttributeValue<'b>>) -> Object<'b> {
        assert_eq!(values.len(), self.slots.len(), "Wrong number of arguments");
        Object { class: self, values }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.is(other)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Type {} @ {:?}", self, self.id())
    }
}

pub fn make_float<'b>(value: f64) -> Object<'b> {
    let t = BUILTIN_TYPES.get("Float").unwrap();
    let v = AttributeValue::Primitive(Primitive::Float(value));
    let i = t.instance(vec![v]);
    i
}

pub fn make_int<'b>(value: BigInt) -> Object<'b> {
    let t = BUILTIN_TYPES.get("Int").unwrap();
    let v = AttributeValue::Primitive(Primitive::Int(value));
    let i = t.instance(vec![v]);
    i
}
