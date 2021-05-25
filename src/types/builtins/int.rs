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

    fn div_f64(&self, rhs: &ObjectRef) -> Result<f64, RuntimeError> {
        let lhs_val = self.value().to_f64().unwrap();
        let rhs_val = if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
            rhs.value().to_f64().unwrap()
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
            *rhs.value()
        } else {
            return Err(RuntimeError::new_type_error(format!(
                "Could not divide {} into Int",
                rhs.class().name()
            )));
        };
        Ok(lhs_val / rhs_val)
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
    make_op!(add, +, "Could not add {} to Int");
    make_op!(sub, -, "Could not subtract {} from Int");

    // Int division *always* returns a Float
    fn div(&self, rhs: ObjectRef) -> RuntimeResult {
        let value = self.div_f64(&rhs)?;
        // FIXME: Class is Int when RHS is Int but should be Float
        Ok(Rc::new(Float::new(rhs.class().clone(), value)))
    }

    // Int *floor* division *always* returns an Int
    fn floor_div(&self, rhs: ObjectRef) -> RuntimeResult {
        let value = self.div_f64(&rhs)?;
        let value = BigInt::from_f64(value).unwrap();
        Ok(Rc::new(Int::new(self.class().clone(), value)))
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
