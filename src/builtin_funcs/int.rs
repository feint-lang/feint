use num_bigint::BigInt;
use num_traits::FromPrimitive;

use crate::types::{Args, CallResult};
use crate::vm::{RuntimeErr, VM};

pub fn new(args: Args, vm: &mut VM) -> CallResult {
    let arg = args.get(0).unwrap();
    let int = if let Some(val) = arg.int_val() {
        vm.ctx.builtins.new_int(val)
    } else if let Some(val) = arg.float_val() {
        vm.ctx.builtins.new_int(BigInt::from_f64(val).unwrap())
    } else if let Some(val) = arg.str_val() {
        vm.ctx.builtins.new_int_from_string(val)
    } else {
        let message = format!("Int new expected string or int; got {arg}");
        return Err(RuntimeErr::new_type_err(message));
    };
    Ok(Some(int))
}
