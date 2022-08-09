use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};

use super::create;
use super::result::{Args, This};

use super::float::Float;
use super::int::Int;

/// Given an option `this` object, return a string representation.
pub fn this_to_str(this: &This) -> String {
    let t = this.clone().unwrap_or_else(create::new_nil);
    let t = t.read().unwrap();
    t.to_string()
}

/// Given a list of args (`ObjectRef`s), return a string joining them
/// together into a comma separated string surround by parens.
pub fn args_to_str(args: &Args) -> String {
    let strings: Vec<String> =
        args.iter().map(|item| format!("{:?}", &*item.read().unwrap())).collect();
    format!("({})", strings.join(", "))
}

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
