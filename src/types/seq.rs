//! Common sequence operations

use num_bigint::BigInt;

use crate::vm::{RuntimeErr, RuntimeObjResult, VM};

use super::gen::{use_arg, use_arg_str};
use super::new;

use super::base::ObjectRef;
use super::result::Args;

pub fn join(items: &[ObjectRef], args: &Args) -> RuntimeObjResult {
    let arg = use_arg!(args, 0);
    let sep = use_arg_str!(join, sep, arg);
    let n_items = items.len();
    let last_i = n_items - 1;

    // XXX: Guessing at average word length
    let capacity = n_items * 5 + ((last_i) * sep.len());
    let mut string = String::with_capacity(capacity);

    for (i, item) in items.iter().enumerate() {
        let item = item.read().unwrap();
        let str = item.to_string();
        string.push_str(&str);
        if i != last_i {
            string.push_str(sep);
        }
    }

    Ok(new::str(string))
}

pub fn map(
    this: &ObjectRef,
    items: &[ObjectRef],
    args: &Args,
    vm: &mut VM,
) -> RuntimeObjResult {
    let map_fn = &args[0];
    let mut results = vec![];

    let f = map_fn.read().unwrap();
    let f = if let Some(f) = f.as_func() {
        f
    } else {
        return Ok(new::arg_err("map/1 expects a function", this.clone()));
    };

    let n_args = if f.has_var_args() { 2 } else { f.arity() };

    for (i, item) in items.iter().enumerate() {
        if n_args == 1 {
            vm.call(map_fn.clone(), vec![item.clone()])?;
        } else {
            vm.call(map_fn.clone(), vec![item.clone(), new::int(i)])?;
        }
        results.push(vm.pop_obj()?);
    }

    Ok(new::tuple(results))
}

pub fn sum(items: &[ObjectRef]) -> RuntimeObjResult {
    let mut sum = new::int(BigInt::from(0));
    for item in items.iter() {
        let a = sum.read().unwrap();
        let b = item.read().unwrap();
        let new_sum = (*a).add(&*b)?;
        drop(a);
        sum = new_sum;
    }
    Ok(sum)
}
