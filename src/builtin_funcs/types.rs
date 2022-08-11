use crate::types::{create, Args, CallResult, This};
use crate::vm::VM;

/// Returns Type
pub fn type_of(_this: This, args: Args, _vm: &mut VM) -> CallResult {
    let arg = args.first().unwrap();
    let arg = arg.read().unwrap();
    Ok(arg.type_obj().clone())
}

/// Returns Int
pub fn obj_id(_this: This, args: Args, _vm: &mut VM) -> CallResult {
    let arg = args.first().unwrap();
    let arg = arg.read().unwrap();
    Ok(create::new_int(arg.id()))
}

/// Get a sorted tuple of the names of the items an object contains.
pub fn items(_this: This, args: Args, _vm: &mut VM) -> CallResult {
    let obj = args.get(0).expect("items() expected object");
    let obj = obj.read().unwrap();
    let obj_ns = obj.namespace();

    let class = obj.class();
    let class = class.read().unwrap();
    let class_ns = class.namespace();

    let mut names: Vec<String> = class_ns.iter().map(|(n, _)| n).cloned().collect();
    names.extend(obj_ns.iter().map(|(n, _)| n).cloned());
    names.sort();
    names.dedup();

    let items = names.iter().map(create::new_str).collect();
    Ok(create::new_tuple(items))
}
