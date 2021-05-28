use std::any::Any;
use std::fmt;

use num_traits::ToPrimitive;

use crate::vm::{RuntimeBoolResult, RuntimeContext, RuntimeError, RuntimeResult};

use super::super::class::TypeRef;
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
    ( $meth:ident, $op:tt, $message:literal, $trunc:literal ) => {
        fn $meth(&self, rhs: ObjectRef, ctx: &RuntimeContext) -> RuntimeResult {
            let value = if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
                *rhs.value()
            } else if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
                rhs.value().to_f64().unwrap()
            } else {
                return Err(RuntimeError::new_type_error(format!($message, rhs.class().name())));
            };
            let mut value = &self.value $op value;
            if $trunc {
                value = value.trunc();
            }
            let value = ctx.builtins.new_float(value);
            Ok(value)
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

    fn is_equal(&self, rhs: ObjectRef, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            Ok(self.is(rhs) || self == rhs)
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
            Ok(eq_int_float(rhs, self))
        } else {
            Err(RuntimeError::new_type_error(format!(
                "Could not compare Float to {} for equality",
                rhs.class().name()
            )))
        }
    }

    fn pow(&self, rhs: ObjectRef, ctx: &RuntimeContext) -> RuntimeResult {
        let exp = if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
            *rhs.value()
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
            rhs.value().to_f64().unwrap()
        } else {
            return Err(RuntimeError::new_type_error(format!(
                "Could not raise Float by {}",
                rhs.class().name()
            )));
        };
        let value = self.value().powf(exp);
        let value = ctx.builtins.new_float(value);
        Ok(value)
    }

    make_op!(modulo, %, "Could not divide {} with Float", false);
    make_op!(mul, *, "Could not multiply {} with Float", false);
    make_op!(div, /, "Could not divide {} into Float", false);
    make_op!(floor_div, /, "Could not divide {} into Float", true); // truncates
    make_op!(add, +, "Could not add {} to Float", false);
    make_op!(sub, -, "Could not subtract {} from Float", false);
}

// Display -------------------------------------------------------------

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.value().fract() == 0.0 {
            write!(f, "{}.0", self.value())
        } else {
            write!(f, "{}", self.value())
        }
    }
}
