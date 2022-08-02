use crate::types::create;
use crate::types::{Args, CallResult, This};
use crate::vm::VM;

/// Returns Type
pub fn type_of(_this: This, args: Args, _vm: &mut VM) -> CallResult {
    let arg = args.first().unwrap();
    Ok(Some(arg.type_obj().clone()))
}

/// Returns Str
pub fn obj_id(_this: This, args: Args, _vm: &mut VM) -> CallResult {
    let arg = args.first().unwrap();
    Ok(Some(create::new_int_from_usize(arg.id())))
}
