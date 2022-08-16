use crate::types::{create, Args, CallResult, ObjectTrait, This};
use crate::vm::{RuntimeErr, VM};

/// Returns Nil
pub fn assert(_this: This, args: Args, _vm: &mut VM) -> CallResult {
    let arg = args.get(0).unwrap();
    let arg = arg.read().unwrap();
    let success = arg.bool_val()?;
    if success {
        Ok(create::new_true())
    } else {
        let var_args = args.get(1).unwrap();
        let var_args = var_args.read().unwrap();
        let var_args = var_args.down_to_tuple().unwrap();
        let msg = if var_args.len() > 0 {
            let msg_arg = var_args.get_item(0)?;
            let msg_arg = msg_arg.read().unwrap();
            if let Some(msg) = msg_arg.get_str_val() {
                msg.to_string()
            } else {
                msg_arg.to_string()
            }
        } else {
            "".to_string()
        };
        Err(RuntimeErr::assertion_failed(msg))
    }
}
