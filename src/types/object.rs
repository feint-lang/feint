use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use num_bigint::BigInt;

use super::{builtins, ObjectError, ObjectErrorKind, Type};
use std::fmt::Formatter;

pub trait Object {
    fn class(&self) -> Rc<Type>;
    fn get_attribute(&self, name: &str) -> Result<&Rc<Object>, ObjectError>;
    fn set_attribute(&mut self, n: &str, n: Rc<Object>) -> Result<(), ObjectError>;

    fn id(&self) -> usize {
        let p = self as *const Self;
        let p = p as *const () as usize;
        p
    }

    fn name(&self) -> String {
        self.class().name().to_owned()
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        if self.class().is(&other.class()) && self.id() == other.id() {
            return true;
        }
        false
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object")
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object")
    }
}

// Fundamentals --------------------------------------------------------

#[derive(Debug)]
pub enum FundamentalObject {
    None(Rc<Type>),
    Bool(Rc<Type>, bool),
    Float(Rc<Type>, f64),
    Int(Rc<Type>, BigInt),
}

impl PartialEq for FundamentalObject {
    fn eq(&self, other: &Self) -> bool {
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

impl FundamentalObject {
    fn is(&self, other: &Self) -> bool {
        self.class().is(&other.class()) && self.id() == other.id()
    }
}

impl Object for FundamentalObject {
    fn class(&self) -> Rc<Type> {
        match self {
            Self::None(class) => class.clone(),
            Self::Bool(class, _) => class.clone(),
            Self::Float(class, _) => class.clone(),
            Self::Int(class, _) => class.clone(),
        }
    }

    fn get_attribute(&self, name: &str) -> Result<&Rc<Object>, ObjectError> {
        Err(ObjectError::new(ObjectErrorKind::AttributeDoesNotExist(name.to_owned())))
    }

    fn set_attribute(
        &mut self,
        name: &str,
        _value: Rc<Object>,
    ) -> Result<(), ObjectError> {
        Err(ObjectError::new(ObjectErrorKind::AttributeCannotBeSet(name.to_owned())))
    }
}

impl fmt::Display for FundamentalObject {
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

pub struct ComplexObject {
    class: Rc<Type>,
    attributes: HashMap<String, Rc<Object>>,
}

impl ComplexObject {
    pub fn new(class: Rc<Type>) -> Self {
        Self { class, attributes: HashMap::new() }
    }

    fn is(&self, other: &Self) -> bool {
        self.class().is(&other.class()) && self.id() == other.id()
    }
}

impl PartialEq for ComplexObject {
    fn eq(&self, other: &Self) -> bool {
        if self.is(other) {
            return true;
        }
        self.attributes == other.attributes
    }
}

impl Object for ComplexObject {
    fn class(&self) -> Rc<Type> {
        self.class.clone()
    }

    fn get_attribute(&self, name: &str) -> Result<&Rc<Object>, ObjectError> {
        if let Some(value) = self.attributes.get(name) {
            return Ok(value);
        }
        Err(ObjectError::new(ObjectErrorKind::AttributeDoesNotExist(name.to_owned())))
    }

    fn set_attribute(
        &mut self,
        name: &str,
        value: Rc<Object>,
    ) -> Result<(), ObjectError> {
        self.attributes.insert(name.to_owned(), value.clone());
        Ok(())
    }
}

impl fmt::Display for ComplexObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.class.name())
    }
}

impl fmt::Debug for ComplexObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object {}", self.class.name())
    }
}
