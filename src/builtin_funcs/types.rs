use crate::types::{Args, CallResult};
use crate::vm::RuntimeContext;

/// Returns Type
pub fn type_of(args: Args, _ctx: &RuntimeContext) -> CallResult {
    let arg = args.first().unwrap();
    Ok(Some(arg.class().clone()))
}

/// Returns Str
pub fn obj_id(args: Args, ctx: &RuntimeContext) -> CallResult {
    let arg = args.first().unwrap();
    Ok(Some(ctx.builtins.new_int(arg.id())))
}
