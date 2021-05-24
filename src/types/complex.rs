//! A complex object may have builtin objects and other custom objects
//! as attributes. This is opposed to fundamental types, like `Bool` and
//! `Float` that wrap Rust primitives.
use std::any::Any;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use super::class::Type;
use super::object::{Object, ObjectExt};
use super::result::{ObjectError, ObjectErrorKind};

pub struct ComplexObject {
    class: Rc<Type>,
    attributes: HashMap<String, Rc<dyn Object>>,
}

impl ComplexObject {
    pub fn new(class: Rc<Type>) -> Self {
        Self { class: class.clone(), attributes: HashMap::new() }
    }
}

impl Object for ComplexObject {
    fn class(&self) -> &Rc<Type> {
        &self.class
    }

    fn get_attribute(&self, name: &str) -> Result<&Rc<dyn Object>, ObjectError> {
        if let Some(value) = self.attributes.get(name) {
            return Ok(value);
        }
        Err(ObjectError::new(ObjectErrorKind::AttributeDoesNotExist(name.to_owned())))
    }

    fn set_attribute(
        &mut self,
        name: &str,
        value: Rc<dyn Object>,
    ) -> Result<(), ObjectError> {
        self.attributes.insert(name.to_owned(), value.clone());
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Binary operations ---------------------------------------------------

impl PartialEq for ComplexObject {
    fn eq(&self, other: &Self) -> bool {
        if self.is(other) {
            return true;
        }
        self.attributes == other.attributes
    }
}

impl PartialEq<dyn Object> for ComplexObject {
    fn eq(&self, rhs: &dyn Object) -> bool {
        if let Some(rhs) = rhs.as_any().downcast_ref::<ComplexObject>() {
            self == rhs
        } else {
            panic!("Could not compare {} to {}", self.class(), rhs.class());
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Debug for ComplexObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object {} @ {}", self, self.id())
    }
}

impl fmt::Display for ComplexObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<String> =
            self.attributes.iter().map(|(n, v)| format!("{}={}", n, v)).collect();
        write!(f, "{}({})", self.class.name(), names.join(", "))
    }
}
