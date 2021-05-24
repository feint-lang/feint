use std::fmt;
use std::ops::{Add, Sub};

use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};

use builtin_object_derive::BuiltinObject;

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
            panic!("Could not add RHS to Int: {:?}", rhs);
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
            panic!("Could not subtract RHS from Int: {:?}", rhs);
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

// impl From<BigInt> for Int {
//     fn from(value: BigInt) -> Self {
//         Int::new(value)
//     }
// }
//
// macro_rules! int_from {
//     ($($T:ty),+) => { $(
//         impl From<$T> for Int {
//             fn from(value: $T) -> Self {
//                 let value = BigInt::from(value);
//                 Int::new(value)
//             }
//         }
//     )+ };
// }
//
// int_from!(i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);
//
// impl From<f32> for Int {
//     fn from(value: f32) -> Self {
//         let value = BigInt::from_f32(value).unwrap();
//         Int::new(value)
//     }
// }
//
// impl From<f64> for Int {
//     fn from(value: f64) -> Self {
//         let value = BigInt::from_f64(value).unwrap();
//         Int::new(value)
//     }
// }
//
// macro_rules! int_from_string {
//     ($($T:ty),+) => { $(
//         impl From<$T> for Int {
//             fn from(value: $T) -> Self {
//                 let value = BigInt::from_str_radix(value.as_ref(), 10).unwrap();
//                 Int::new(value)
//             }
//         }
//     )+ };
// }
//
// int_from_string!(&str, String, &String);
