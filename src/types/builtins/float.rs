use std::fmt;

use builtin_object_derive::BuiltinObject;

use super::cmp::eq_int_float;
use super::int::Int;

/// Built in 64-bit float type
#[derive(Debug, PartialEq, BuiltinObject)]
pub struct Float {
    value: f64,
}

impl Float {
    pub fn new(value: f64) -> Self {
        let instance = Self { value };
        instance.class().clone();
        instance
    }

    pub fn value(&self) -> &f64 {
        &self.value
    }

    /// Is this Float equal to the specified Int?
    pub fn eq_int(&self, int: &Int) -> bool {
        eq_int_float(int, self)
    }
}

macro_rules! float_from {
    ($($T:ty),+) => { $(
        impl From<$T> for Float {
            fn from(value: $T) -> Self {
                let value = value as f64;
                Float::new(value)
            }
        }
    )+ };
}

float_from!(f32, f64, i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);

macro_rules! float_from_string {
    ($($T:ty),+) => { $(
        impl From<$T> for Float {
            fn from(value: $T) -> Self {
                let value = value.parse::<f64>().unwrap();
                Float::new(value)
            }
        }
    )+ };
}

float_from_string!(&str, String, &String);

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
