use crate::types::{Args, CallResult};
use crate::vm::RuntimeContext;

/// Returns Nil
pub fn print(args: Args, _ctx: &RuntimeContext) -> CallResult {
    for arg in args {
        let arg = arg.lock().unwrap();
        print!("{arg}");
    }
    println!();
    Ok(None)
}
