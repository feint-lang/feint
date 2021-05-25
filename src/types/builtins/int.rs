use std::any::Any;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;

use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};

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

    /// Is this Int equal to the specified Float?
    pub fn eq_float(&self, float: &Float) -> bool {
        eq_int_float(self, float)
    }
}

macro_rules! make_op {
    ( $meth:ident, $op:tt, $message:literal ) => {
        fn $meth(&self, rhs: ObjectRef) -> ObjectRef {
            if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
                // XXX: Return Int
                let value = self.value() $op rhs.value();
                Rc::new(Int::new(self.class.clone(), value))
            } else if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
                // XXX: Return Float
                let value = self.value().to_f64().unwrap() $op rhs.value();
                Rc::new(Float::new(rhs.class().clone(), value))
            } else {
                panic!($message, rhs.class());
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

    fn is_equal(&self, rhs: ObjectRef) -> bool {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            return self.is(rhs) || self == rhs;
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
            return self.eq_float(rhs);
        }
        panic!("Could not compare Int to {}", rhs.class());
    }

    make_op!(mul, *, "Could not multiply {:?} with Int");
    make_op!(div, /, "Could not divide {:?} into Int");
    make_op!(add, +, "Could not add {:?} to Int");
    make_op!(sub, -, "Could not subtract {:?} from Int");
}

// Display -------------------------------------------------------------

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
