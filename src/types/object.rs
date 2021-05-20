use std::fmt;

use num_bigint::BigInt;

use super::{ErrorKind, Type};

pub trait ObjectTrait {
    // fn class(&self) -> &Type;

    /// The unique ID of the object.
    fn id(&self) -> *const Self {
        self as *const Self
    }

    /// Is this object the other object?
    fn is(&self, other: &Self) -> bool {
        self.id() == other.id()
    }

    fn name(&self) -> String;
    fn is_equal(&self, other: &Self) -> bool;
    fn to_string(&self) -> String;
}

// Primitive -----------------------------------------------------------

#[derive(Debug)]
pub enum Primitive {
    None,
    Bool(bool),
    Float(f64),
    Int(BigInt),
}

impl PartialEq for Primitive {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}

impl ObjectTrait for Primitive {
    // fn class(&self) -> &Type {
    //     &self.class
    // }

    fn name(&self) -> String {
        match self {
            Self::None => "None".to_owned(),
            Self::Bool(_) => "Bool".to_owned(),
            Self::Float(_) => "Float".to_owned(),
            Self::Int(_) => "Int".to_owned(),
        }
    }

    fn is_equal(&self, other: &Self) -> bool {
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

    fn to_string(&self) -> String {
        match self {
            Self::None => "None".to_owned(),
            Self::Bool(bool) => bool.to_string(),
            Self::Float(value) => value.to_string(),
            Self::Int(value) => value.to_string(),
        }
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

// Object --------------------------------------------------------------

pub struct Object<'a> {
    pub class: &'a Type,
    pub values: Vec<AttributeValue<'a>>,
}

impl PartialEq for Object<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}

impl Object<'_> {
    pub fn get_attribute(&self, name: &str) -> Result<&AttributeValue, ErrorKind> {
        // FIXME: Seems suboptimal to iterate every time rather than
        //        doing a map lookup.
        if let Some(i) = self.class.slots.iter().position(|n| n == name) {
            return Ok(&self.values[i]);
        }
        Err(ErrorKind::AttributeDoesNotExistError(name.to_owned()))
    }
}

impl ObjectTrait for Object<'_> {
    // fn class(&self) -> &Type {
    //     self.class
    // }

    fn is(&self, other: &Self) -> bool {
        self.class.id() == other.class.id() && self.id() == other.id()
    }

    fn name(&self) -> String {
        format!("{}", self.class.name)
    }

    fn is_equal(&self, other: &Self) -> bool {
        self.is(other) || (self.values == other.values)
    }

    fn to_string(&self) -> String {
        format!("{}({:?})", self.class.name, self.values)
    }
}

impl fmt::Display for Object<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Debug for Object<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object {} @ {:?} = {:?}", self.class, self.id(), self.values)
    }
}

// AttributeValue ------------------------------------------------------

#[derive(PartialEq)]
pub enum AttributeValue<'a> {
    Primitive(Primitive),
    Object(Object<'a>),
}

impl AttributeValue<'_> {
    pub fn to_string(&self) -> String {
        let string = match self {
            Self::Primitive(Primitive::None) => "None".to_string(),
            Self::Primitive(Primitive::Bool(value)) => format!("{}", value),
            Self::Primitive(Primitive::Float(value)) => format!("{}", value),
            Self::Primitive(Primitive::Int(value)) => format!("{}", value),
            Self::Object(object) => format!("{:?}", object),
        };
        string
    }
}

impl fmt::Display for AttributeValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
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
