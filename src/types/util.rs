use num_bigint::BigInt;
use num_traits::FromPrimitive;

use crate::types::float::Float;
use crate::types::int::Int;

/// Compare Int and Float for equality.
pub fn eq_int_float(int: &Int, float: &Float) -> bool {
    let float_val = float.value();
    if float_val.fract() == 0.0 {
        let int_val = int.value();
        let float_as_int = BigInt::from_f64(*float_val).unwrap();
        float_as_int == *int_val
    } else {
        false
    }
}
