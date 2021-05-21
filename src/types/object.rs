use std::any::Any;
use std::collections::HashMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::rc::Rc;
use std::sync::Arc;

use num_bigint::BigInt;

use super::builtins::{Bool, Float, Int};
use super::result::{ObjectError, ObjectErrorKind};
use super::types::Type;

pub trait Object {
    fn class(&self) -> Arc<Type>;

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

    fn id(&self) -> usize {
        let p = self as *const Self;
        let p = p as *const () as usize;
        p
    }

    fn name(&self) -> String {
        self.class().name().to_owned()
    }

    fn as_any(&self) -> &dyn Any;
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        // This should catch None and Bool, since they're singletons
        // (or will be).
        if self.class().is(&other.class()) && self.id() == other.id() {
            return true;
        }

        if let Some(a) = self.as_any().downcast_ref::<Bool>() {
            if let Some(b) = other.as_any().downcast_ref::<Bool>() {
                return false;
            }
        }

        if let Some(a) = self.as_any().downcast_ref::<Float>() {
            if let Some(b) = other.as_any().downcast_ref::<Float>() {
                return a == b;
            }
        }

        if let Some(a) = self.as_any().downcast_ref::<Int>() {
            if let Some(b) = other.as_any().downcast_ref::<Int>() {
                return a == b;
            }
        }

        if let Some(a) = self.as_any().downcast_ref::<Float>() {
            if let Some(b) = other.as_any().downcast_ref::<Int>() {
                return a.eq_int(b);
            }
        }

        if let Some(a) = self.as_any().downcast_ref::<Int>() {
            if let Some(b) = other.as_any().downcast_ref::<Float>() {
                return a.eq_float(b);
            }
        }

        if let Some(a) = self.as_any().downcast_ref::<ComplexObject>() {
            if let Some(b) = other.as_any().downcast_ref::<ComplexObject>() {
                return a == b;
            }
        }

        panic!("Could not compare {:?} and {:?}", self, other);
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Object")
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Object")
    }
}

// ---------------------------------------------------------------------

/// A complex object may have builtin objects and other custom objects
/// as attributes.
pub struct ComplexObject {
    class: Arc<Type>,
    attributes: HashMap<String, Rc<Object>>,
}

impl ComplexObject {
    pub fn new(class: Arc<Type>) -> Self {
        Self { class, attributes: HashMap::new() }
    }

    fn is(&self, other: &Self) -> bool {
        self.class().is(&other.class()) && self.id() == other.id()
    }
}

impl Object for ComplexObject {
    fn class(&self) -> Arc<Type> {
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PartialEq for ComplexObject {
    fn eq(&self, other: &Self) -> bool {
        println!("COMPLEX EQ");
        if self.is(other) {
            return true;
        }
        println!("COMPLEX EQ NOT IS");
        self.attributes == other.attributes
    }
}

impl Debug for ComplexObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Object {} @ {}", self, self.id())
    }
}

impl Display for ComplexObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let names: Vec<String> = self
            .attributes
            .iter()
            .map(|(n, v)| format!("{}={}", n, v.to_string()))
            .collect();
        write!(f, "{}({})", self.class.name(), names.join(", "))
    }
}
