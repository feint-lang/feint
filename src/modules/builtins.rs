//! Builtins Module
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::types::{new, Module, Namespace, ObjectRef, ObjectTrait};
use crate::vm::RuntimeErr;

use crate::types::bool::BOOL_TYPE;
use crate::types::bound_func::BOUND_FUNC_TYPE;
use crate::types::builtin_func::BUILTIN_FUNC_TYPE;
use crate::types::class::TYPE_TYPE;
use crate::types::closure::CLOSURE_TYPE;
use crate::types::error::ERROR_TYPE;
use crate::types::float::FLOAT_TYPE;
use crate::types::func::FUNC_TYPE;
use crate::types::int::INT_TYPE;
use crate::types::list::LIST_TYPE;
use crate::types::map::MAP_TYPE;
use crate::types::module::MODULE_TYPE;
use crate::types::nil::NIL_TYPE;
use crate::types::str::STR_TYPE;
use crate::types::tuple::TUPLE_TYPE;

pub static BUILTINS: Lazy<new::obj_ref_t!(Module)> = Lazy::new(|| {
    let entries: Vec<(&str, ObjectRef)> = vec![
        ("Type", TYPE_TYPE.clone()),
        ("Bool", BOOL_TYPE.clone()),
        ("BoundFunc", BOUND_FUNC_TYPE.clone()),
        ("BuiltinFunc", BUILTIN_FUNC_TYPE.clone()),
        ("Closure", CLOSURE_TYPE.clone()),
        ("Error", ERROR_TYPE.clone()),
        ("Func", FUNC_TYPE.clone()),
        ("Float", FLOAT_TYPE.clone()),
        ("Int", INT_TYPE.clone()),
        ("List", LIST_TYPE.clone()),
        ("Map", MAP_TYPE.clone()),
        ("Module", MODULE_TYPE.clone()),
        ("Nil", NIL_TYPE.clone()),
        ("Str", STR_TYPE.clone()),
        ("Tuple", TUPLE_TYPE.clone()),
        (
            // Print representation of zero or more objects to stdout.
            //
            // Args:
            //    objects?: ObjectRef[]
            "print",
            new::builtin_func("print", None, &[""], |_, args, _| {
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
            }),
        ),
        (
            // Check condition and return error if false.
            //
            // Args:
            //     condition: Bool
            //     message?: Any
            "assert",
            new::builtin_func("assert", None, &["assertion", ""], |_, args, _| {
                let arg = args.get(0).unwrap();
                let arg = arg.read().unwrap();
                let success = arg.bool_val()?;
                if success {
                    Ok(new::true_())
                } else {
                    let var_args = args.get(1).unwrap();
                    let var_args = var_args.read().unwrap();
                    let var_args = var_args.down_to_tuple().unwrap();
                    let msg = if var_args.is_empty() {
                        "".to_string()
                    } else {
                        let msg_arg = var_args.get_item(0)?;
                        let msg_arg = msg_arg.read().unwrap();
                        if let Some(msg) = msg_arg.get_str_val() {
                            msg.to_string()
                        } else {
                            msg_arg.to_string()
                        }
                    };
                    Err(RuntimeErr::assertion_failed(msg))
                }
            }),
        ),
        (
            // Get the type of an object.
            //
            // Args:
            //
            //    object: ObjectRef
            "type",
            new::builtin_func("type", None, &["object"], |_, args, _| {
                let arg = args.first().unwrap();
                let arg = arg.read().unwrap();
                Ok(arg.type_obj().clone())
            }),
        ),
        (
            // Get the ID of an object.
            //
            // Args:
            //
            //    object: ObjectRef
            "id",
            new::builtin_func("id", None, &["object"], |_, args, _| {
                let arg = args.first().unwrap();
                let arg = arg.read().unwrap();
                Ok(new::int(arg.id()))
            }),
        ),
    ];

    new::builtin_module("builtins", Namespace::with_entries(&entries))
});
