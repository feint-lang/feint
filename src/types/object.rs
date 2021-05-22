use std::any::Any;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use super::builtins::{Bool, Float, Int};
use super::class::Type;
use super::complex::ComplexObject;
use super::result::{ObjectError, ObjectErrorKind};

/// Represents an instance of some type (AKA "class").
pub trait Object {
    fn class(&self) -> Arc<Type>;

    fn get_attribute(&self, name: &str) -> Result<&Rc<dyn Object>, ObjectError> {
        Err(ObjectError::new(ObjectErrorKind::AttributeDoesNotExist(name.to_owned())))
    }

    fn set_attribute(
        &mut self,
        name: &str,
        _value: Rc<dyn Object>,
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

/// Methods that aren't "object safe"
pub trait ObjectExt: Object {
    fn is(&self, other: &Self) -> bool {
        self.class().is(&other.class()) && self.id() == other.id()
    }
}

impl<T: Object + ?Sized> ObjectExt for T {}

impl PartialEq for dyn Object {
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

impl fmt::Display for dyn Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object")
    }
}

impl fmt::Debug for dyn Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object")
    }
}

// ---------------------------------------------------------------------
