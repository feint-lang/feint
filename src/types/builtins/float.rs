use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;

use num_traits::ToPrimitive;

use builtin_object_derive::BuiltinObject;

use super::super::class::Type;
use super::super::object::Object;

use super::cmp::eq_int_float;
use super::int::Int;

/// Built in 64-bit float type
#[derive(Debug, PartialEq, BuiltinObject)]
pub struct Float {
    class: Rc<Type>,
    value: f64,
}

impl Float {
    pub fn new(class: Rc<Type>, value: f64) -> Self {
        Self { class: class.clone(), value }
    }

    pub fn value(&self) -> &f64 {
        &self.value
    }

    /// Is this Float equal to the specified Int?
    pub fn eq_int(&self, int: &Int) -> bool {
        eq_int_float(int, self)
    }
}

// Equality ------------------------------------------------------------

impl PartialEq<dyn Object> for Float {
    fn eq(&self, rhs: &dyn Object) -> bool {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
            self == rhs
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
            self.eq_int(rhs)
        } else {
            panic!("Could not compare Float to {}", rhs.class());
        }
    }
}

// Binary operations ---------------------------------------------------

macro_rules! make_op {
    ( $trait:ident, $meth:ident, $op:tt, $message:literal ) => {
        impl<'a> $trait<Rc<dyn Object>> for &'a Float {
            type Output = Rc<dyn Object>;
            fn $meth(self, rhs: Rc<dyn Object>) -> Self::Output {
                let value = if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
                    *rhs.value()
                } else if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
                    rhs.value().to_f64().unwrap()
                } else {
                    panic!($message, rhs.class());
                };
                let value = &self.value $op value;
                Rc::new(Float::new(self.class.clone(), value))
            }
        }
    };
}

make_op!(Mul, mul, *, "Could not multiply {:?} with Float");
make_op!(Div, div, /, "Could not divide {:?} into Float");
make_op!(Add, add, +, "Could not add {:?} to Float");
make_op!(Sub, sub, -, "Could not subtract {:?} from Float");

// Display -------------------------------------------------------------

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
