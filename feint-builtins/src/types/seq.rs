//! Common sequence operations

use num_bigint::BigInt;

use feint_code_gen::{use_arg, use_arg_str};

use super::base::ObjectRef;
use super::{new, Args};

pub fn each(this: &ObjectRef, items: &[ObjectRef], args: &Args) -> ObjectRef {
    if items.is_empty() {
        return new::nil();
    }

    let each_fn = &args[0];
    let f = each_fn.read().unwrap();
    let n_args = if let Some(f) = f.as_func() {
        if f.has_var_args() {
            2
        } else {
            f.arity()
        }
    } else {
        return new::arg_err("each/1 expects a function", this.clone());
    };

    // for (i, item) in items.iter().enumerate() {
    //     let each = each_fn.clone();
    //     let item = item.clone();
    //     if n_args == 1 {
    //         vm.call(each, vec![item])?;
    //     } else {
    //         vm.call(each, vec![item, new::int(i)])?;
    //     }
    // }

    new::nil()
}

pub fn has(items: &[ObjectRef], args: &Args) -> ObjectRef {
    if items.is_empty() {
        return new::bool(false);
    }
    let member = use_arg!(args, 0);
    for item in items.iter() {
        if member.is_equal(&*item.read().unwrap()) {
            return new::bool(true);
        }
    }
    new::bool(false)
}

pub fn join(items: &[ObjectRef], args: &Args) -> ObjectRef {
    if items.is_empty() {
        return new::empty_str();
    }

    let n_items = items.len();
    let last_i = n_items - 1;
    let arg = use_arg!(args, 0);
    let sep = use_arg_str!(join, sep, arg);

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

    new::str(string)
}

pub fn map(this: &ObjectRef, items: &[ObjectRef], args: &Args) -> ObjectRef {
    if items.is_empty() {
        return new::empty_tuple();
    }

    let map_fn = &args[0];
    let f = map_fn.read().unwrap();
    let n_args = if let Some(f) = f.as_func() {
        if f.has_var_args() {
            2
        } else {
            f.arity()
        }
    } else {
        return new::arg_err("map/1 expects a function", this.clone());
    };

    let mut results = vec![];
    // for (i, item) in items.iter().enumerate() {
    //     let map = map_fn.clone();
    //     let item = item.clone();
    //     if n_args == 1 {
    //         vm.call(map, vec![item])?;
    //     } else {
    //         vm.call(map, vec![item, new::int(i)])?;
    //     }
    //     results.push(vm.pop_obj()?);
    // }

    new::tuple(results)
}

pub fn sum(items: &[ObjectRef]) -> ObjectRef {
    let mut sum = new::int(BigInt::from(0));
    for item in items.iter() {
        sum = {
            let a = sum.read().unwrap();
            let b = item.read().unwrap();
            if let Some(new_sum) = (*a).add(&*b) {
                new_sum
            } else {
                return new::type_err("Could not add object to sum", item.clone());
            }
        }
    }
    sum
}
