use std::any::Any;
use std::fmt;
use std::rc::Rc;

use crate::vm::{RuntimeError, RuntimeResult, VM};

use super::builtins::{Bool, Float, Int, Nil};
use super::class::{Type, TypeRef};
use super::complex::ComplexObject;
use super::result::{ObjectError, ObjectErrorKind};

pub type ObjectRef = Rc<dyn Object>;

/// Represents an instance of some type (AKA "class").
pub trait Object {
    fn class(&self) -> &TypeRef;
    fn as_any(&self) -> &dyn Any;

    fn id(&self) -> usize {
        let p = self as *const Self;
        let p = p as *const () as usize;
        p
    }

    fn name(&self) -> String {
        self.class().name().to_owned()
    }

    fn is_equal(&self, _rhs: ObjectRef, _vm: &VM) -> Result<bool, RuntimeError> {
        // This should catch Bool (when both true or both false) and Nil
        // (always), since they're singletons.
        Err(RuntimeError::new_type_error(format!(
            "is_equal not implemented for type: {}",
            self.class().name()
        )))
    }

    // Binary operations -----------------------------------------------

    fn mul(&self, _rhs: ObjectRef, _vm: &VM) -> RuntimeResult {
        Err(RuntimeError::new_type_error(format!(
            "mul not implemented for type: {}",
            self.class().name()
        )))
    }

    fn div(&self, _rhs: ObjectRef, _vm: &VM) -> RuntimeResult {
        Err(RuntimeError::new_type_error(format!(
            "div not implemented for type: {}",
            self.class().name()
        )))
    }

    fn floor_div(&self, _rhs: ObjectRef, _vm: &VM) -> RuntimeResult {
        Err(RuntimeError::new_type_error(format!(
            "floor_div not implemented for type: {}",
            self.class().name()
        )))
    }

    fn add(&self, _rhs: ObjectRef, _vm: &VM) -> RuntimeResult {
        Err(RuntimeError::new_type_error(format!(
            "add not implemented for type: {}",
            self.class().name()
        )))
    }

    fn sub(&self, _rhs: ObjectRef, _vm: &VM) -> RuntimeResult {
        Err(RuntimeError::new_type_error(format!(
            "sub not implemented for type: {}",
            self.class().name()
        )))
    }

    // Attributes ------------------------------------------------------

    fn get_attribute(&self, name: &str) -> Result<&ObjectRef, ObjectError> {
        Err(ObjectError::new(ObjectErrorKind::AttributeDoesNotExist(name.to_owned())))
    }

    fn set_attribute(
        &mut self,
        name: &str,
        _value: ObjectRef,
    ) -> Result<(), ObjectError> {
        Err(ObjectError::new(ObjectErrorKind::AttributeCannotBeSet(name.to_owned())))
    }
}

// Object extensions ---------------------------------------------------

/// Methods that aren't "object safe"
pub trait ObjectExt: Object {
    fn is(&self, other: &Self) -> bool {
        self.class().is(&other.class()) && self.id() == other.id()
    }
}

impl<T: Object + ?Sized> ObjectExt for T {}

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
