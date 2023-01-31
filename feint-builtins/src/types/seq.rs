//! Common sequence operations

use num_bigint::BigInt;

use feint_code_gen::{use_arg, use_arg_str};

use super::base::ObjectRef;
use super::{new, Args};

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
