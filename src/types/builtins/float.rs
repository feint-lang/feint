use std::any::Any;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;

use num_traits::ToPrimitive;

use crate::vm::{RuntimeError, RuntimeResult};

use super::super::class::{Type, TypeRef};
use super::super::object::{Object, ObjectExt, ObjectRef};

use super::cmp::eq_int_float;
use super::int::Int;

/// Built in 64-bit float type
#[derive(Debug, PartialEq)]
pub struct Float {
    class: TypeRef,
    value: f64,
}

impl Float {
    pub fn new(class: TypeRef, value: f64) -> Self {
        Self { class: class.clone(), value }
    }

    pub fn value(&self) -> &f64 {
        &self.value
    }
}

macro_rules! make_op {
    ( $meth:ident, $op:tt, $message:literal ) => {
        fn $meth(&self, rhs: ObjectRef) -> RuntimeResult {
            let value = if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
                *rhs.value()
            } else if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
                rhs.value().to_f64().unwrap()
            } else {
                return Err(RuntimeError::new_type_error(format!($message, rhs.class().name())));
            };
            let value = &self.value $op value;
            Ok(Rc::new(Float::new(self.class.clone(), value)))
        }
    };
}

impl Object for Float {
    fn class(&self) -> &TypeRef {
        &self.class
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_equal(&self, rhs: ObjectRef) -> Result<bool, RuntimeError> {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            Ok(self.is(rhs) || self == rhs)
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
            Ok(eq_int_float(rhs, self))
        } else {
            Err(RuntimeError::new_type_error(format!(
                "Could not compare Float to {}",
                rhs.class().name()
            )))
        }
    }

    make_op!(mul, *, "Could not multiply {} with Float");
    make_op!(div, /, "Could not divide {} into Float");
    make_op!(add, +, "Could not add {} to Float");
    make_op!(sub, -, "Could not subtract {} from Float");
}

// Display -------------------------------------------------------------

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
