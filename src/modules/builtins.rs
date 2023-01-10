//! Builtins Module
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::types::{new, Args, Module, Namespace, ObjectRef};

use crate::types::always::ALWAYS_TYPE;
use crate::types::bool::BOOL_TYPE;
use crate::types::bound_func::BOUND_FUNC_TYPE;
use crate::types::builtin_func::BUILTIN_FUNC_TYPE;
use crate::types::class::TYPE_TYPE;
use crate::types::closure::CLOSURE_TYPE;
use crate::types::err::ERR_TYPE;
use crate::types::err_type::ERR_TYPE_TYPE;
use crate::types::file::FILE_TYPE;
use crate::types::float::FLOAT_TYPE;
use crate::types::func::FUNC_TYPE;
use crate::types::gen::use_arg;
use crate::types::int::INT_TYPE;
use crate::types::list::LIST_TYPE;
use crate::types::map::MAP_TYPE;
use crate::types::module::MODULE_TYPE;
use crate::types::nil::NIL_TYPE;
use crate::types::str::STR_TYPE;
use crate::types::tuple::TUPLE_TYPE;

use crate::util::check_args;
use crate::vm::{RuntimeErr, RuntimeObjResult};

pub static BUILTINS: Lazy<new::obj_ref_t!(Module)> = Lazy::new(|| {
    let entries: Vec<(&str, ObjectRef)> = vec![
        ("$doc", new::str("Builtins module")),
        ("Type", TYPE_TYPE.clone()),
        ("Always", ALWAYS_TYPE.clone()),
        ("Bool", BOOL_TYPE.clone()),
        ("BoundFunc", BOUND_FUNC_TYPE.clone()),
        ("BuiltinFunc", BUILTIN_FUNC_TYPE.clone()),
        ("Closure", CLOSURE_TYPE.clone()),
        ("Err", ERR_TYPE.clone()),
        ("ErrType", ERR_TYPE_TYPE.clone()),
        ("File", FILE_TYPE.clone()),
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
            "help",
            new::builtin_func(
                "help",
                None,
                &["obj"],
                "Print the docstring for an object.

                Args:
                   object: Any",
                |_, args, _| {
                    if let Err(err) = check_args("help", &args, false, 1, Some(1)) {
                        return Ok(err);
                    }
                    let arg = use_arg!(args, 0);
                    let obj = args[0].clone();
                    let doc = arg.get_attr("$doc", obj.clone());
                    let doc = doc.read().unwrap();
                    if doc.is_err() {
                        let obj = obj.read().unwrap();
                        let type_obj_ref = obj.type_obj();
                        let type_obj = type_obj_ref.read().unwrap();
                        let doc = type_obj.get_attr("$doc", type_obj_ref.clone());
                        let doc = doc.read().unwrap();
                        if doc.is_err() {
                            eprintln!(
                                concat!(
                                    "Could not read $doc attribute of object {:?} ",
                                    "or its type {}"
                                ),
                                &*obj, type_obj
                            );
                        } else {
                            println!("{doc}");
                        }
                    } else {
                        println!("{doc}");
                    }
                    Ok(new::empty_str())
                },
            ),
        ),
        (
            "print",
            new::builtin_func(
                "print",
                None,
                &[""],
                "Print representation of zero or more objects to stdout.

                Args:
                   objects?: ObjectRef[]",
                |_, args, _| print(&args, false),
            ),
        ),
        (
            "print_err",
            new::builtin_func(
                "print_err",
                None,
                &[""],
                "Print representation of zero or more objects to stderr.

                Args:
                    objects?: ObjectRef[]",
                |_, args, _| print(&args, true),
            ),
        ),
        (
            "assert",
            new::builtin_func(
                "assert",
                None,
                &["assertion", ""],
                "Check condition and return error if false.
            
                Args:
                    condition: Bool
                    message?: Any
                    throw?: Bool = false
                
                Returns:
                    true: if the assertion succeeded
                    Err: if the assertion failed and throw unset
                    RuntimeErr: if the assertion failed and throw set",
                |_, args, _| {
                    let result = check_args("assert()", &args, true, 1, Some(3));
                    if let Err(err) = result {
                        return Ok(err);
                    }
                    let (_, n_var_args, var_args_obj) = result.unwrap();

                    let arg = args.get(0).unwrap();
                    let arg = arg.read().unwrap();
                    let success = arg.bool_val()?;

                    if success {
                        return Ok(new::true_());
                    }

                    let var_args = var_args_obj.read().unwrap();

                    let msg = if n_var_args == 0 {
                        "".to_string()
                    } else {
                        let msg_arg = var_args.get_item(0, var_args_obj.clone());
                        let msg_arg = msg_arg.read().unwrap();
                        msg_arg.to_string()
                    };

                    if n_var_args == 2 {
                        let throw_arg = var_args.get_item(1, var_args_obj.clone());
                        let throw_arg = throw_arg.read().unwrap();
                        if throw_arg.bool_val()? {
                            return Err(RuntimeErr::assertion_failed(msg));
                        }
                    }

                    Ok(new::assertion_err(msg, new::nil()))
                },
            ),
        ),
        (
            "type",
            new::builtin_func(
                "type",
                None,
                &["object"],
                "Get the type of an object.

                Args:
                    object: ObjectRef",
                |_, args, _| {
                    let arg = args.first().unwrap();
                    let arg = arg.read().unwrap();
                    Ok(arg.type_obj().clone())
                },
            ),
        ),
        (
            "id",
            new::builtin_func(
                "id",
                None,
                &["object"],
                "Get the ID of an object.

                Args:
                    object: ObjectRef",
                |_, args, _| {
                    let arg = args.first().unwrap();
                    let arg = arg.read().unwrap();
                    Ok(new::int(arg.id()))
                },
            ),
        ),
    ];

    new::builtin_module("builtins", Namespace::with_entries(&entries))
});

fn print(args: &Args, err: bool) -> RuntimeObjResult {
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
            if err {
                eprint!("{arg}{sep}");
            } else {
                print!("{arg}{sep}");
            }
        }
    }
    if err {
        eprintln!()
    } else {
        println!()
    }
    return Ok(new::nil());
}
