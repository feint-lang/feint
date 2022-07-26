use crate::types::{Args, CallResult};
use crate::vm::RuntimeContext;

/// Returns Type
pub fn type_of(args: Args, ctx: &RuntimeContext) -> CallResult {
    let arg = args.first().unwrap();
    let name = arg.class().name();
    let class = ctx.get_type(name);
    Ok(Some(class))
}

/// Returns Str
pub fn obj_id(args: Args, ctx: &RuntimeContext) -> CallResult {
    let arg = args.first().unwrap();
    Ok(Some(ctx.builtins.new_int(arg.id())))
}
