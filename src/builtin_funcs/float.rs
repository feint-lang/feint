use num_traits::ToPrimitive;

use crate::types::{create, Args, CallResult, This};
use crate::vm::{RuntimeErr, VM};

pub fn new(_this: This, args: Args, _vm: &mut VM) -> CallResult {
    let arg = args.get(0).unwrap();
    let arg = arg.read().unwrap();
    let float = if let Some(val) = arg.get_float_val() {
        create::new_float(val)
    } else if let Some(val) = arg.get_int_val() {
        create::new_float(val.to_f64().unwrap())
    } else if let Some(val) = arg.get_str_val() {
        create::new_float_from_string(val)
    } else {
        let message = format!("Float new expected string or float; got {arg}");
        return Err(RuntimeErr::new_type_err(message));
    };
    Ok(Some(float))
}
