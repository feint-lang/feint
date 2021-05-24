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

// Binary operations ---------------------------------------------------

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

impl<'a> Mul<Rc<dyn Object>> for &'a Int {
    type Output = Rc<dyn Object>;
    fn mul(self, rhs: Rc<dyn Object>) -> Self::Output {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
            let value = self.value() * rhs.value();
            Rc::new(Int::new(self.class.clone(), value))
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
            let value = self.value().to_f64().unwrap() * rhs.value();
            Rc::new(Float::new(rhs.class().clone(), value))
        } else {
            panic!("Could not multiply {:?} with Int", rhs.class());
        }
    }
}

impl<'a> Div<Rc<dyn Object>> for &'a Int {
    type Output = Rc<dyn Object>;
    fn div(self, rhs: Rc<dyn Object>) -> Self::Output {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
            let value = self.value() / rhs.value();
            Rc::new(Int::new(self.class.clone(), value))
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
            let value = self.value().to_f64().unwrap() / rhs.value();
            Rc::new(Float::new(rhs.class().clone(), value))
        } else {
            panic!("Could not divide {:?} into Int", rhs.class());
        }
    }
}

impl<'a> Add<Rc<dyn Object>> for &'a Int {
    type Output = Rc<dyn Object>;
    fn add(self, rhs: Rc<dyn Object>) -> Self::Output {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
            let value = self.value() + rhs.value();
            Rc::new(Int::new(self.class.clone(), value))
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
            let value = self.value().to_f64().unwrap() + rhs.value();
            Rc::new(Float::new(rhs.class().clone(), value))
        } else {
            panic!("Could not add {:?} to Int", rhs.class());
        }
    }
}

impl<'a> Sub<Rc<dyn Object>> for &'a Int {
    type Output = Rc<dyn Object>;
    fn sub(self, rhs: Rc<dyn Object>) -> Self::Output {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Int>() {
            let value = self.value() - rhs.value();
            Rc::new(Int::new(self.class.clone(), value))
        } else if let Some(rhs) = rhs.as_any().downcast_ref::<Float>() {
            let value = self.value().to_f64().unwrap() - rhs.value();
            Rc::new(Float::new(rhs.class().clone(), value))
        } else {
            panic!("Could not subtract {:?} from Int", rhs.class());
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
