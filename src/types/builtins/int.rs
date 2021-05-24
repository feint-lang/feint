use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;

use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};

use builtin_object_derive::BuiltinObject;

use super::super::class::Type;
use super::super::object::Object;

use super::cmp::eq_int_float;
use super::float::Float;

/// Built in integer type
#[derive(Debug, PartialEq, BuiltinObject)]
pub struct Int {
    class: Rc<Type>,
    value: BigInt,
}

impl Int {
    pub fn new(class: Rc<Type>, value: BigInt) -> Self {
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

// Equality ------------------------------------------------------------

impl PartialEq<dyn Object> for Int {
    fn eq(&self, rhs: &dyn Object) -> bool {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
            self == rhs
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
            self.eq_float(rhs)
        } else {
            panic!("Could not compare Int to {}", rhs.class());
        }
    }
}

// Binary operations ---------------------------------------------------

macro_rules! make_op {
    ( $trait:ident, $meth:ident, $op:tt, $message:literal ) => {
        impl<'a> $trait<Rc<dyn Object>> for &'a Int {
            type Output = Rc<dyn Object>;
            fn $meth(self, rhs: Rc<dyn Object>) -> Self::Output {
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
        }
    };
}

make_op!(Mul, mul, *, "Could not multiply {:?} with Int");
make_op!(Div, div, /, "Could not divide {:?} into Int");
make_op!(Add, add, +, "Could not add {:?} to Int");
make_op!(Sub, sub, -, "Could not subtract {:?} from Int");

// Display -------------------------------------------------------------

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
