use crate::types::{create, Args, CallResult, This};
use crate::vm::{RuntimeErr, VM};

pub fn map(this: This, args: Args, vm: &mut VM) -> CallResult {
    let this = this.expect("Expected this");
    let this = this.read().unwrap();
    if let Some(tuple) = this.down_to_tuple() {
        let map_fn = args.get(0).unwrap();
        let map_fn = map_fn.read().unwrap();
        for (i, item) in tuple.iter().enumerate() {
            let i = create::new_int(i);
            map_fn.call(None, vec![item.clone(), i], vm)?;
        }
        Ok(create::new_nil())
    } else {
        let message = format!(
            "map() expected a tuple as its first arg; got {:?}",
            this.type_obj()
        );
        Err(RuntimeErr::new_type_err(message))
    }
}
