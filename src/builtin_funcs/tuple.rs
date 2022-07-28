use crate::types::{Args, CallResult};
use crate::vm::{RuntimeErr, VM};

pub fn map(args: Args, vm: &mut VM) -> CallResult {
    let this = args.get(0).unwrap();
    if let Some(this) = this.as_tuple() {
        let map_fn = args.get(1).unwrap();
        for (i, item) in this.items().iter().enumerate() {
            let i = vm.ctx.builtins.new_int(i);
            map_fn.call(vec![item.clone(), i], vm)?;
        }
        Ok(None)
    } else {
        Err(RuntimeErr::new_type_err("Builtin map function expected a tuple"))
    }
}
