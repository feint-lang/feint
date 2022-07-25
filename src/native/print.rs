use crate::types::ObjectRef;
use crate::vm::RuntimeContext;

use super::result::CallResult;

pub fn print(args: Vec<ObjectRef>, _ctx: &RuntimeContext) -> CallResult {
    for arg in args {
        print!("{arg}");
    }
    println!();
    Ok(None)
}
