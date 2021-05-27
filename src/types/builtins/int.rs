use std::any::Any;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;

use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};

use crate::vm::{RuntimeContext, RuntimeError, RuntimeResult};

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

    // Cast both LHS and RHS to f64 and divide them
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
        fn $meth(&self, rhs: ObjectRef, ctx: &RuntimeContext) -> RuntimeResult {
            if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
                // XXX: Return Int
                let value = self.value() $op rhs.value();
                let value = ctx.builtins.new_int(value);
                Ok(value)
            } else if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
                // XXX: Return Float
                let value = self.value().to_f64().unwrap() $op rhs.value();
                let value = ctx.builtins.new_float(value);
                Ok(value)
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

    fn is_equal(
        &self,
        rhs: ObjectRef,
        _ctx: &RuntimeContext,
    ) -> Result<bool, RuntimeError> {
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
    fn raise(&self, rhs: ObjectRef, ctx: &RuntimeContext) -> RuntimeResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
            // XXX: Return Int
            let base = self.value();
            let exp = rhs.value().to_u32().unwrap();
            let value = base.pow(exp);
            let value = ctx.builtins.new_int(value);
            Ok(value)
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
            // XXX: Return Float
            let base = self.value().to_f64().unwrap();
            let exp = *rhs.value();
            let value = base.powf(exp);
            let value = ctx.builtins.new_float(value);
            Ok(value)
        } else {
            Err(RuntimeError::new_type_error(format!(
                "Could not raise Int by {}",
                rhs.class().name()
            )))
        }
    }

    make_op!(modulo, %, "Could not divide {} with Int");
    make_op!(mul, *, "Could not multiply {} with Int");
    make_op!(add, +, "Could not add {} to Int");
    make_op!(sub, -, "Could not subtract {} from Int");

    // Int division *always* returns a Float
    fn div(&self, rhs: ObjectRef, ctx: &RuntimeContext) -> RuntimeResult {
        let value = self.div_f64(&rhs)?;
        let value = ctx.builtins.new_float(value);
        Ok(value)
    }

    // Int *floor* division *always* returns an Int
    fn floor_div(&self, rhs: ObjectRef, ctx: &RuntimeContext) -> RuntimeResult {
        let value = self.div_f64(&rhs)?;
        let value = BigInt::from_f64(value).unwrap();
        let value = ctx.builtins.new_int(value);
        Ok(value)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}
