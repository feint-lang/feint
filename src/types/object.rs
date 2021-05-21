use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use num_bigint::BigInt;

use super::{builtins, ErrorKind, Type};

pub trait ObjectTrait {
    fn class(&self) -> Rc<Type>;

    /// The unique ID of the object.
    fn id(&self) -> *const Self {
        self as *const Self
    }

    fn name(&self) -> String {
        self.class().name().to_owned()
    }

    /// Is this object the other object?
    fn is(&self, other: &Self) -> bool {
        self.class().id() == other.class().id() && self.id() == other.id()
    }

    fn is_equal(&self, other: &Self) -> bool;
}

// Fundamentals --------------------------------------------------------

#[derive(Debug)]
pub enum Fundamental {
    None(Rc<Type>),
    Bool(Rc<Type>, bool),
    Float(Rc<Type>, f64),
    Int(Rc<Type>, BigInt),
}

impl PartialEq for Fundamental {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}

impl ObjectTrait for Fundamental {
    fn class(&self) -> Rc<Type> {
        match self {
            Self::None(class) => class.clone(),
            Self::Bool(class, _) => class.clone(),
            Self::Float(class, _) => class.clone(),
            Self::Int(class, _) => class.clone(),
        }
    }

    fn is_equal(&self, other: &Self) -> bool {
        if self.is(other) {
            return true;
        }
        match (self, other) {
            (Self::Float(_, a), Self::Int(_, b)) => {
                // FIXME: This probably isn't the right way to do this
                a.fract() == 0.0 && BigInt::from(*a as i128) == *b
            }
            (Self::Int(_, a), Self::Float(_, b)) => {
                b.fract() == 0.0 && BigInt::from(*b as i128) == *a
            }
            // TODO: I'm not sure these cases need to be checked since
            //       the `self == other` check above should catch them.
            (Self::None(_), Self::None(_)) => true,
            (Self::Bool(_, a), Self::Bool(_, b)) => a == b,
            (Self::Float(_, a), Self::Float(_, b)) => a == b,
            (Self::Int(_, a), Self::Int(_, b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for Fundamental {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            Self::None(_) => "None".to_owned(),
            Self::Bool(_, bool) => bool.to_string(),
            Self::Float(_, value) => value.to_string(),
            Self::Int(_, value) => value.to_string(),
        };
        write!(f, "{}", string)
    }
}

// Object --------------------------------------------------------------

pub struct Object {
    class: Rc<Type>,
    attributes: HashMap<String, Rc<Attribute>>,
}

impl Object {
    pub fn new(class: Rc<Type>) -> Self {
        Self { class, attributes: HashMap::new() }
    }

    pub fn set_attribute(&mut self, name: &str, value: Rc<Attribute>) {
        self.attributes.insert(name.to_owned(), value);
    }

    pub fn get_attribute(&self, name: &str) -> Result<&Rc<Attribute>, ErrorKind> {
        if let Some(value) = self.attributes.get(name) {
            return Ok(value);
        }
        Err(ErrorKind::AttributeDoesNotExistError(name.to_owned()))
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}

impl ObjectTrait for Object {
    fn class(&self) -> Rc<Type> {
        self.class.clone()
    }

    fn is_equal(&self, other: &Self) -> bool {
        self.is(other) || (self.attributes == other.attributes)
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({:?})", self.class.name(), self.attributes)
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Object {} @ {:?} = {:?}",
            self.class.name(),
            self.id(),
            self.attributes
        )
    }
}

// Attribute -----------------------------------------------------------

#[derive(PartialEq)]
pub enum Attribute {
    Fundamental(Fundamental),
    Object(Object),
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            Self::Fundamental(Fundamental::None(_)) => "None".to_string(),
            Self::Fundamental(Fundamental::Bool(_, value)) => value.to_string(),
            Self::Fundamental(Fundamental::Float(_, value)) => value.to_string(),
            Self::Fundamental(Fundamental::Int(_, value)) => value.to_string(),
            Self::Object(object) => object.to_string(),
        };
        write!(f, "{}", string)
    }
}

impl fmt::Debug for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            Self::Fundamental(Fundamental::None(_)) => "None".to_string(),
            Self::Fundamental(Fundamental::Bool(_, value)) => {
                format!("Bool({})", value)
            }
            Self::Fundamental(Fundamental::Float(_, value)) => {
                format!("Float({})", value)
            }
            Self::Fundamental(Fundamental::Int(_, value)) => format!("Int({})", value),
            Self::Object(object) => format!("{:?}", object),
        };
        write!(f, "{}", string)
    }
}
