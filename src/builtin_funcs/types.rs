use crate::types::{Args, CallResult};
use crate::vm::VM;

/// Returns Type
pub fn type_of(args: Args, _vm: &mut VM) -> CallResult {
    let arg = args.first().unwrap();
    Ok(Some(arg.class().clone()))
}

/// Returns Str
pub fn obj_id(args: Args, vm: &mut VM) -> CallResult {
    let arg = args.first().unwrap();
    Ok(Some(vm.ctx.builtins.new_int(arg.id())))
}
