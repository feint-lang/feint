use num_bigint::BigInt;
use num_traits::FromPrimitive;

use crate::types::{create, Args, CallResult, This};
use crate::vm::{RuntimeErr, VM};

pub fn new(_this: This, args: Args, _vm: &mut VM) -> CallResult {
    let arg = args.get(0).unwrap();
    let arg = arg.read().unwrap();
    let int = if let Some(val) = arg.get_int_val() {
        create::new_int(val)
    } else if let Some(val) = arg.get_float_val() {
        create::new_int(BigInt::from_f64(val).unwrap())
    } else if let Some(val) = arg.get_str_val() {
        create::new_int_from_string(val)
    } else {
        let message = format!("Int new expected string or int; got {arg}");
        return Err(RuntimeErr::new_type_err(message));
    };
    Ok(int)
}
