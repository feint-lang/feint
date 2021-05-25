use std::any::Any;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;

use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};

use crate::vm::{RuntimeError, RuntimeResult};

use super::super::class::{Type, TypeRef};
use super::super::object::{Object, ObjectExt, ObjectRef};

use super::cmp::eq_int_float;
use super::float::Float;

/// Built in integer type
#[derive(Debug, PartialEq)]
pub struct Int {
    class: TypeRef,
    value: BigInt,
}

impl Int {
    pub fn new(class: TypeRef, value: BigInt) -> Self {
        Self { class: class.clone(), value }
    }

    pub fn value(&self) -> &BigInt {
        &self.value
    }
}

macro_rules! make_op {
    ( $meth:ident, $op:tt, $message:literal ) => {
        fn $meth(&self, rhs: ObjectRef) -> RuntimeResult {
            if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
                // XXX: Return Int
                let value = self.value() $op rhs.value();
                Ok(Rc::new(Int::new(self.class.clone(), value)))
            } else if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
                // XXX: Return Float
                let value = self.value().to_f64().unwrap() $op rhs.value();
                Ok(Rc::new(Float::new(rhs.class().clone(), value)))
            } else {
                Err(RuntimeError::new_type_error(format!($message, rhs.class().name())))
            }
        }
    };
}

impl Object for Int {
    fn class(&self) -> &TypeRef {
        &self.class
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_equal(&self, rhs: ObjectRef) -> Result<bool, RuntimeError> {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            Ok(self.is(rhs) || self == rhs)
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
            Ok(eq_int_float(self, rhs))
        } else {
            Err(RuntimeError::new_type_error(format!(
                "Could not compare Int to {}",
                rhs.class().name()
            )))
        }
    }

    make_op!(mul, *, "Could not multiply {} with Int");
    make_op!(div, /, "Could not divide {} into Int");
    make_op!(add, +, "Could not add {} to Int");
    make_op!(sub, -, "Could not subtract {} from Int");
}

// Display -------------------------------------------------------------

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
