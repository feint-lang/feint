use std::any::Any;
use std::fmt;
use std::rc::Rc;

use super::builtins::{Bool, Float, Int, Nil};
use super::class::Type;
use super::complex::ComplexObject;
use super::result::{ObjectError, ObjectErrorKind};

macro_rules! binop {
    ( $lhs:ident, $op:tt, $rhs:ident, $($LHS:ty),+ ) => { $(
        if let Some(lhs) = $lhs.as_any().downcast_ref::<$LHS>() {
            return lhs $op $rhs;
        }
    )+ };
}

/// Represents an instance of some type (AKA "class").
pub trait Object {
    fn class(&self) -> &Rc<Type>;

    fn add(&self, rhs: Rc<dyn Object>) -> Rc<dyn Object> {
        binop!(self, +, rhs, Float, Int);
        panic!("Could not add items: {} + {}", self.class(), rhs.class());
    }

    fn sub(&self, rhs: Rc<dyn Object>) -> Rc<dyn Object> {
        binop!(self, -, rhs, Float, Int);
        panic!("Could not subtract items: {} - {}", self.class(), rhs.class());
    }

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

// Object extensions ---------------------------------------------------

/// Methods that aren't "object safe"
pub trait ObjectExt: Object {
    fn is(&self, other: &Self) -> bool {
        self.class().is(&other.class()) && self.id() == other.id()
    }
}

impl<T: Object + ?Sized> ObjectExt for T {}

// Equality ------------------------------------------------------------

/// Defer to concrete type by downcasting.
impl PartialEq for dyn Object {
    fn eq(&self, other: &Self) -> bool {
        // This should catch Bool (when both true or both false) and Nil
        // (always), since they're singletons.
        if self.is(other) {
            return true;
        }
        binop!(self, ==, other, Bool, Float, Int, ComplexObject);
        panic!("Could not compare {:?} and {:?}", self, other);
    }
}

// Display -------------------------------------------------------------

/// Downcast Object to concrete type/object and display that.
macro_rules! write_instance {
    ( $f:ident, $a:ident, $($A:ty),+ ) => { $(
        if let Some(a) = $a.as_any().downcast_ref::<$A>() {
            return write!($f, "{}", a);
        }
    )+ };
}

impl fmt::Display for dyn Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_instance!(f, self, Nil, Bool, Float, Int, ComplexObject);
        write!(f, "{}()", self.class())
    }
}

impl fmt::Debug for dyn Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} object @ {:?} -> {}", self.class(), self.id(), self)
    }
}
