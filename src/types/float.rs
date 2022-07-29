//! Float type (64 bit)
use std::any::Any;
use std::fmt;

use num_traits::ToPrimitive;

use crate::vm::{RuntimeBoolResult, RuntimeContext, RuntimeErr, RuntimeObjResult};

use super::builtin_types::BUILTIN_TYPES;
use super::class::TypeRef;
use super::int::Int;
use super::object::{Object, ObjectExt};
use super::util::{eq_int_float, gt_int_float, lt_int_float};

pub struct Float {
    value: f64,
}

impl Float {
    pub fn new(value: f64) -> Self {
        Self { value }
    }

    pub fn value(&self) -> &f64 {
        &self.value
    }
}

macro_rules! make_op {
    ( $meth:ident, $op:tt, $message:literal, $trunc:literal ) => {
        fn $meth(&self, rhs: &dyn Object, ctx: &RuntimeContext) -> RuntimeObjResult {
            let value = if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
                *rhs.value()
            } else if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
                rhs.value().to_f64().unwrap()
            } else {
                return Err(RuntimeErr::new_type_err(format!($message, rhs.type_name())));
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
        BUILTIN_TYPES.get("Float").unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn negate(&self, ctx: &RuntimeContext) -> RuntimeObjResult {
        Ok(ctx.builtins.new_float(-self.value()))
    }

    fn as_bool(&self, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        Ok(if *self.value() == 0.0 { false } else { true })
    }

    fn is_equal(&self, rhs: &dyn Object, _ctx: &RuntimeContext) -> bool {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            self.is(rhs) || self.value() == rhs.value()
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
            eq_int_float(rhs, self)
        } else {
            false
        }
    }

    fn less_than(&self, rhs: &dyn Object, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            Ok(self.value() < rhs.value())
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
            Ok(lt_int_float(rhs, self))
        } else {
            Err(RuntimeErr::new_type_err(format!(
                "Could not compare {} to {}: <",
                self.class(),
                rhs.class()
            )))
        }
    }

    fn greater_than(
        &self,
        rhs: &dyn Object,
        _ctx: &RuntimeContext,
    ) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            Ok(self.value() > rhs.value())
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
            Ok(gt_int_float(rhs, self))
        } else {
            Err(RuntimeErr::new_type_err(format!(
                "Could not compare {} to {}: >",
                self.class(),
                rhs.class()
            )))
        }
    }

    fn pow(&self, rhs: &dyn Object, ctx: &RuntimeContext) -> RuntimeObjResult {
        let exp = if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            *rhs.value()
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
            rhs.value().to_f64().unwrap()
        } else {
            return Err(RuntimeErr::new_type_err(format!(
                "Could not raise {} by {}",
                self.class(),
                rhs.class()
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

impl fmt::Debug for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
