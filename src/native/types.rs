use crate::types::ObjectRef;
use crate::vm::RuntimeContext;

use super::result::CallResult;

pub fn type_of(args: Vec<ObjectRef>, ctx: &RuntimeContext) -> CallResult {
    let arg = args.first().unwrap();
    Ok(Some(ctx.builtins.new_string(arg.class().name())))
}
