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

// Binary operations ---------------------------------------------------

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

fn rhs_to_f64(rhs: Rc<dyn Object>) -> f64 {
    if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
        *rhs.value()
    } else if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
        rhs.value().to_f64().unwrap()
    } else {
        panic!("Could not convert {} to f64", rhs.class());
    }
}

impl<'a> Mul<Rc<dyn Object>> for &'a Float {
    type Output = Rc<dyn Object>;
    fn mul(self, rhs: Rc<dyn Object>) -> Self::Output {
        let value = &self.value * rhs_to_f64(rhs);
        Rc::new(Float::new(self.class.clone(), value))
    }
}

impl<'a> Div<Rc<dyn Object>> for &'a Float {
    type Output = Rc<dyn Object>;
    fn div(self, rhs: Rc<dyn Object>) -> Self::Output {
        let value = &self.value / rhs_to_f64(rhs);
        Rc::new(Float::new(self.class.clone(), value))
    }
}

impl<'a> Add<Rc<dyn Object>> for &'a Float {
    type Output = Rc<dyn Object>;
    fn add(self, rhs: Rc<dyn Object>) -> Self::Output {
        let value = &self.value + rhs_to_f64(rhs);
        Rc::new(Float::new(self.class.clone(), value))
    }
}

impl<'a> Sub<Rc<dyn Object>> for &'a Float {
    type Output = Rc<dyn Object>;
    fn sub(self, rhs: Rc<dyn Object>) -> Self::Output {
        let value = &self.value - rhs_to_f64(rhs);
        Rc::new(Float::new(self.class.clone(), value))
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
