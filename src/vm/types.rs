use std::collections::HashMap;
use std::fmt;

use lazy_static::lazy_static;

use num_bigint::BigInt;
use std::ops::Index;

lazy_static! {
    static ref TYPES: HashMap<&'static str, Type> = {
        let mut types = HashMap::new();
        types.insert("None", Type::None);
        types.insert("Bool", Type::Bool);
        types.insert("Float", Type::Float);
        types.insert("Int", Type::Int);
        types
    };
}

pub enum Type {
    None,
    Bool,
    Float,
    Int,
    Custom(String, Vec<String>, Vec<Type>),
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.is(other)
    }
}

impl Type {
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

    pub fn name(&self) -> &str {
        ""
    }

    /// Create a custom type comprising other types.
    pub fn custom(name: &str, slots: Vec<&str>, types: Vec<Type>) -> Type {
        assert_eq!(slots.len(), types.len());
        let name = name.to_owned();
        let slots = slots.iter().map(|s| (*s).to_owned()).collect();
        Type::Custom(name, slots, types)
    }

    /// Constructors
    pub fn instance<'a>(&'a self, values: Vec<AttributeValue<'a>>) -> Object<'a> {
        Object { class: self, values }
    }

    pub fn make_float<'a>(value: f64) -> Object<'a> {
        Self::Float.instance(vec![AttributeValue::Primitive(Primitive::Float(value))])
    }

    pub fn make_int<'a>(value: BigInt) -> Object<'a> {
        Self::Int.instance(vec![AttributeValue::Primitive(Primitive::Int(value))])
    }
}

// Objects -------------------------------------------------------------

pub trait ObjectTrait {
    /// The unique ID of the object.
    fn id(&self) -> *const Self {
        self as *const Self
    }

    /// Is this object the other object?
    fn is(&self, other: &Self) -> bool {
        self.id() == other.id()
    }

    fn equal(&self, other: &Self) -> bool;
}

#[derive(Debug)]
pub enum Primitive {
    None,
    Bool(u8),
    Float(f64),
    Int(BigInt),
}

impl PartialEq for Primitive {
    fn eq(&self, other: &Self) -> bool {
        self.equal(other)
    }
}

impl ObjectTrait for Primitive {
    fn equal(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Float(a), Self::Int(b)) => {
                // FIXME: This probably isn't the right way to do this
                a.fract() == 0.0 && BigInt::from(*a as i128) == *b
            }
            (Self::Int(a), Self::Float(b)) => {
                b.fract() == 0.0 && BigInt::from(*b as i128) == *a
            }
            // TODO: I'm not sure these cases need to be checked since
            //       the `self == other` check above should catch them.
            (Self::None, Self::None) => true,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a == b,
            (Self::Int(a), Self::Int(b)) => a == b,
            _ => false,
        }
    }
}

pub struct Object<'a> {
    class: &'a Type,
    values: Vec<AttributeValue<'a>>,
}

impl PartialEq for Object<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.equal(other)
    }
}

impl Object<'_> {
    fn get_attribute(&self, name: &str) -> Result<&AttributeValue, ErrorKind> {
        if let Type::Custom(_, slots, _) = self.class {
            // FIXME: Seems suboptimal to iterate every time rather than
            //        doing a map lookup.
            if let Some(i) = slots.iter().position(|n| n == name) {
                return Ok(&self.values[i]);
            }
        }
        Err(ErrorKind::AttributeDoesNotExistError(name.to_owned()))
    }
}

impl ObjectTrait for Object<'_> {
    fn is(&self, other: &Self) -> bool {
        self.class.id() == other.class.id() && self.id() == other.id()
    }

    fn equal(&self, other: &Self) -> bool {
        self.is(other) || (self.values == other.values)
    }
}

#[derive(PartialEq)]
pub enum AttributeValue<'a> {
    Primitive(Primitive),
    Object(Object<'a>),
}

// Result and error types ----------------------------------------------

pub enum ErrorKind {
    TypeError,
    AttributeDoesNotExistError(String),
}

// Display impls -------------------------------------------------------

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type {} @ {:?}", self, self.id())
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::None => "None".to_string(),
            Self::Bool => "Bool".to_string(),
            Self::Float => "Float".to_string(),
            Self::Int => "Int".to_string(),
            Self::Custom(name, slots, types) => {
                let items: Vec<String> = slots
                    .iter()
                    .zip(types)
                    .map(|(name, class)| format!("{}: {}", name, class))
                    .collect();
                format!("{}({})", name, items.join(", "))
            }
        };
        write!(f, "{}", string)
    }
}

impl fmt::Debug for Object<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object {} @ {:?} = {:?}", self.class, self.id(), self.values)
    }
}

impl fmt::Display for Object<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({:?})", self.class, self.values)
    }
}

impl fmt::Debug for AttributeValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::Primitive(Primitive::None) => "None".to_string(),
            Self::Primitive(Primitive::Bool(value)) => format!("Bool({})", value),
            Self::Primitive(Primitive::Float(value)) => format!("Float({})", value),
            Self::Primitive(Primitive::Int(value)) => format!("Int({})", value),
            Self::Object(object) => format!("{:?}", object),
        };
        write!(f, "{}", string)
    }
}

impl fmt::Display for AttributeValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Self::Primitive(Primitive::None) => "None".to_string(),
            Self::Primitive(Primitive::Bool(value)) => format!("{}", value),
            Self::Primitive(Primitive::Float(value)) => format!("{}", value),
            Self::Primitive(Primitive::Int(value)) => format!("{}", value),
            Self::Object(object) => format!("{:?}", object),
        };
        write!(f, "{}", string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float() {
        let float1 = Type::make_float(0.0);
        let float2 = Type::make_float(0.0);
        let float3 = Type::make_float(1.0);
        assert_eq!(float1.class.id(), float2.class.id());
        assert_eq!(float2.class.id(), float3.class.id());
        assert_ne!(float1.id(), float2.id());
        assert_ne!(float2.id(), float3.id());
        assert!(float1.equal(&float2));
        assert!(!float1.equal(&float3));
    }

    #[test]
    fn test_compare_float_to_int() {
        let float = Type::make_float(0.0);
        let int = Type::make_int(BigInt::from(0));
        assert_eq!(float, int);
    }

    #[test]
    fn test_custom() {
        let type_1 = Type::custom("Custom1", vec!["value"], vec![Type::Int]);
        let value_1 = AttributeValue::Object(Type::make_int(BigInt::from(0)));
        let obj_1 = type_1.instance(vec![value_1]);

        let type_2 = Type::custom("Custom2", vec!["value"], vec![Type::Int]);
        let value_2 = AttributeValue::Object(Type::make_int(BigInt::from(0)));
        let obj_2 = type_2.instance(vec![value_2]);

        assert_ne!(type_1, type_2);
        assert_eq!(obj_1, obj_2);

        if let Ok(value) = obj_1.get_attribute("value") {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
