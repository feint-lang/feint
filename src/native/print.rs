use crate::types::{Args, CallResult};
use crate::vm::RuntimeContext;

pub fn print(args: Args, _ctx: &RuntimeContext) -> CallResult {
    for arg in args {
        print!("{arg}");
    }
    println!();
    Ok(None)
}
