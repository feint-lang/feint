use std::any::Any;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::Rc;

use num_traits::ToPrimitive;

use super::super::class::Type;
use super::super::object::{Object, ObjectExt};

use super::cmp::eq_int_float;
use super::int::Int;

/// Built in 64-bit float type
#[derive(Debug, PartialEq)]
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

macro_rules! make_op {
    ( $meth:ident, $op:tt, $message:literal ) => {
        fn $meth(&self, rhs: Rc<dyn Object>) -> Rc<dyn Object> {
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
    };
}

impl Object for Float {
    fn class(&self) -> &Rc<Type> {
        &self.class
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_equal(&self, rhs: Rc<dyn Object>) -> bool {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            return self.is(rhs) || self == rhs;
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
            return self.eq_int(rhs);
        }
        panic!("Could not compare Float to {}", rhs.class());
    }

    make_op!(mul, *, "Could not multiply {:?} with Float");
    make_op!(div, /, "Could not divide {:?} into Float");
    make_op!(add, +, "Could not add {:?} to Float");
    make_op!(sub, -, "Could not subtract {:?} from Float");
}

// Display -------------------------------------------------------------

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
