use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};

use super::float::Float;
use super::int::Int;

/// Compare Int and Float for equality.
pub fn eq_int_float(int: &Int, float: &Float) -> bool {
    let float_val = float.value();
    if float_val.fract() == 0.0 {
        let int_val = int.value();
        let float_as_int = BigInt::from_f64(*float_val).unwrap();
        *int_val == float_as_int
    } else {
        false
    }
}

/// Compare Int and Float for less than.
pub fn lt_int_float(int: &Int, float: &Float) -> bool {
    let int_as_float = int.value().to_f64().unwrap();
    int_as_float < *float.value()
}

/// Compare Int and Float for greater than.
pub fn gt_int_float(int: &Int, float: &Float) -> bool {
    let int_as_float = int.value().to_f64().unwrap();
    int_as_float > *float.value()
}
