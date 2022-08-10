use crate::types::{create, Args, CallResult, This};
use crate::vm::VM;

/// Returns Type
pub fn type_of(_this: This, args: Args, _vm: &mut VM) -> CallResult {
    let arg = args.first().unwrap();
    let arg = arg.read().unwrap();
    Ok(arg.type_obj().clone())
}

/// Returns Str
pub fn obj_id(_this: This, args: Args, _vm: &mut VM) -> CallResult {
    let arg = args.first().unwrap();
    let arg = arg.read().unwrap();
    Ok(create::new_int(arg.id()))
}
