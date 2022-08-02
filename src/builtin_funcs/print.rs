use crate::types::{Args, CallResult, This};
use crate::vm::VM;

/// Returns Nil
pub fn print(this: This, args: Args, _vm: &mut VM) -> CallResult {
    assert!(this.is_none());
    let num_args = args.len();
    if num_args > 0 {
        let last = num_args - 1;
        let mut sep = " ";
        for (i, arg) in args.iter().enumerate() {
            let arg = arg.read().unwrap();
            if i == last {
                sep = "";
            }
            print!("{arg}{sep}");
        }
    }
    println!();
    Ok(None)
}
