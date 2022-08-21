use crate::types::{new, Args, CallResult, This};
use crate::vm::VM;

/// Returns Nil
pub fn print(_this: This, args: Args, _vm: &mut VM) -> CallResult {
    let items = args.get(0).unwrap();
    let obj_ref = items.read().unwrap();
    let tuple = obj_ref.down_to_tuple().unwrap();
    let count = tuple.len();
    if count > 0 {
        let last = count - 1;
        let mut sep = " ";
        for (i, arg) in tuple.iter().enumerate() {
            let arg = arg.read().unwrap();
            if i == last {
                sep = "";
            }
            print!("{arg}{sep}");
        }
    }
    println!();
    Ok(new::nil())
}
