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

/// Used to compare two instance of the *same* type.
macro_rules! compare_instances {
    ($a:ident, $b:ident, $name:ident) => {
        if let Some(a) = $a.as_any().downcast_ref::<$name>() {
            if let Some(b) = $b.as_any().downcast_ref::<$name>() {
                return a == b;
            }
        }
    };
}

/// Used to compare two instance of *different* types.
macro_rules! compare_instances_of_different_types {
    ($a:ident, $b:ident, $a_type:ident, $b_type:ident, $a_meth:ident) => {
        if let Some(a) = $a.as_any().downcast_ref::<$a_type>() {
            if let Some(b) = $b.as_any().downcast_ref::<$b_type>() {
                return a.$a_meth(b);
            }
        }
    };
}

impl PartialEq for dyn Object {
    fn eq(&self, other: &Self) -> bool {
        // This should catch Bool (when both true or both false) and Nil
        // (always), since they're singletons.
        if self.is(other) {
            return true;
        }
        compare_instances!(self, other, Bool);
        compare_instances!(self, other, Float);
        compare_instances!(self, other, Int);
        compare_instances_of_different_types!(self, other, Float, Int, eq_int);
        compare_instances_of_different_types!(self, other, Int, Float, eq_float);
        compare_instances!(self, other, ComplexObject);
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
