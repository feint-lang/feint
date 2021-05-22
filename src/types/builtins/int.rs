use std::fmt;

use num_bigint::BigInt;
use num_traits::{FromPrimitive, Num};

use builtin_object_derive::BuiltinObject;

use super::cmp::eq_int_float;
use super::float::Float;

/// Built in integer type
#[derive(Debug, PartialEq, BuiltinObject)]
pub struct Int {
    value: BigInt,
}

impl Int {
    pub fn value(&self) -> &BigInt {
        &self.value
    }

    /// Is this Int equal to the specified Float?
    pub fn eq_float(&self, float: &Float) -> bool {
        eq_int_float(self, float)
    }
}

impl From<BigInt> for Int {
    fn from(value: BigInt) -> Self {
        Int { value }
    }
}

macro_rules! int_from {
    ($($T:ty),+) => { $(
        impl From<$T> for Int {
            fn from(value: $T) -> Self {
                Int { value: BigInt::from(value) }
            }
        }
    )+ };
}

int_from!(i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);

impl From<f32> for Int {
    fn from(value: f32) -> Self {
        let value = BigInt::from_f32(value).unwrap();
        Int { value }
    }
}

impl From<f64> for Int {
    fn from(value: f64) -> Self {
        let value = BigInt::from_f64(value).unwrap();
        Int { value }
    }
}

macro_rules! int_from_string {
    ($($T:ty),+) => { $(
        impl From<$T> for Int {
            fn from(value: $T) -> Self {
                let value = BigInt::from_str_radix(value.as_ref(), 10).unwrap();
                Int { value }
            }
        }
    )+ };
}

int_from_string!(&str, String, &String);

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
