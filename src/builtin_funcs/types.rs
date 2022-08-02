use crate::types::create;
use crate::types::{Args, CallResult, This};
use crate::vm::VM;

/// Returns Type
pub fn type_of(this: This, args: Args, _vm: &mut VM) -> CallResult {
    assert!(this.is_none());
    let arg = args.first().unwrap();
    let arg = arg.read().unwrap();
    Ok(Some(arg.type_obj().clone()))
}

/// Returns Str
pub fn obj_id(this: This, args: Args, _vm: &mut VM) -> CallResult {
    assert!(this.is_none());
    let arg = args.first().unwrap();
    let arg = arg.read().unwrap();
    Ok(Some(create::new_int(arg.id())))
}
