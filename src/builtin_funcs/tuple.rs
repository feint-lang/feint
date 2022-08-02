use crate::types::{create, Args, CallResult, This};
use crate::vm::{RuntimeErr, VM};

pub fn map(this: This, args: Args, vm: &mut VM) -> CallResult {
    let this = this.expect("Expected this");
    let orig_this = this.clone();
    if let Some(this) = this.down_to_tuple() {
        let map_fn = args.get(0).unwrap();
        for (i, item) in this.iter().enumerate() {
            let i = create::new_int_from_usize(i);
            map_fn.call(Some(orig_this.clone()), vec![item.clone(), i], vm)?;
        }
        Ok(None)
    } else {
        let message =
            format!("Tuple.map() expected a tuple as its first arg; got {this:?}");
        Err(RuntimeErr::new_type_err(message))
    }
}
