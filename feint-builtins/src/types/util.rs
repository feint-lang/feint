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

/// Check Float < Int
pub fn float_lt_int(lhs_float: &Float, rhs_int: &Int) -> bool {
    let rhs_as_float = rhs_int.value().to_f64().unwrap();
    *lhs_float.value() < rhs_as_float
}

/// Check Int < Float
pub fn int_lt_float(lhs_int: &Int, rhs_float: &Float) -> bool {
    let lhs_as_float = lhs_int.value().to_f64().unwrap();
    lhs_as_float < *rhs_float.value()
}

/// Check Float > Int
pub fn float_gt_int(lhs_float: &Float, rhs_int: &Int) -> bool {
    let rhs_as_float = rhs_int.value().to_f64().unwrap();
    *lhs_float.value() > rhs_as_float
}

/// Check Int > Float
pub fn int_gt_float(lhs_int: &Int, rhs_float: &Float) -> bool {
    let lhs_as_float = lhs_int.value().to_f64().unwrap();
    lhs_as_float > *rhs_float.value()
}
