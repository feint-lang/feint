use crate::types::{Args, CallResult};

use crate::vm::{RuntimeErr, VM};
use num_traits::ToPrimitive;

pub fn new(args: Args, vm: &mut VM) -> CallResult {
    let arg = args.get(0).unwrap();
    let float = if let Some(val) = arg.get_float_val() {
        vm.ctx.builtins.new_float(val)
    } else if let Some(val) = arg.get_int_val() {
        vm.ctx.builtins.new_float(val.to_f64().unwrap())
    } else if let Some(val) = arg.get_str_val() {
        vm.ctx.builtins.new_float_from_string(val)
    } else {
        let message = format!("Float new expected string or float; got {arg}");
        return Err(RuntimeErr::new_type_err(message));
    };
    Ok(Some(float))
}
