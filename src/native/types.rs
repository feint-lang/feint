use crate::types::{Args, CallResult};
use crate::vm::RuntimeContext;

pub fn type_of(args: Args, ctx: &RuntimeContext) -> CallResult {
    let arg = args.first().unwrap();
    Ok(Some(ctx.builtins.new_string(arg.class().name())))
}
