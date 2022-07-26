use crate::types::{Args, CallResult};
use crate::vm::RuntimeContext;

pub fn type_of(args: Args, ctx: &RuntimeContext) -> CallResult {
    let arg = args.first().unwrap();
    // TODO: Return the actual type here rather than its name. Can't do
    //       that now because types aren't objects.
    Ok(Some(ctx.builtins.new_string(arg.class().name())))
}

pub fn obj_id(args: Args, ctx: &RuntimeContext) -> CallResult {
    let arg = args.first().unwrap();
    Ok(Some(ctx.builtins.new_int(arg.id())))
}
