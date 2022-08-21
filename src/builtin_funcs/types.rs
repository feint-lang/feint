use crate::types::{new, Args, CallResult, ObjectRef};
use crate::vm::VM;

/// Returns Type
pub fn type_of(_this: ObjectRef, args: Args, _vm: &mut VM) -> CallResult {
    let arg = args.first().unwrap();
    let arg = arg.read().unwrap();
    Ok(arg.type_obj().clone())
}

/// Returns Int
pub fn obj_id(_this: ObjectRef, args: Args, _vm: &mut VM) -> CallResult {
    let arg = args.first().unwrap();
    let arg = arg.read().unwrap();
    Ok(new::int(arg.id()))
}
