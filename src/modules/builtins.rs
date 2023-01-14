//! Builtins Module
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::types::{
    gen::{use_arg, use_arg_bool},
    new, Module, Namespace,
};
use crate::util::check_args;
use crate::vm::RuntimeErr;

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
use crate::types::int::INT_TYPE;
use crate::types::list::LIST_TYPE;
use crate::types::map::MAP_TYPE;
use crate::types::module::MODULE_TYPE;
use crate::types::nil::NIL_TYPE;
use crate::types::str::STR_TYPE;
use crate::types::tuple::TUPLE_TYPE;

pub static BUILTINS: Lazy<new::obj_ref_t!(Module)> = Lazy::new(|| {
    let ns = Namespace::with_entries(&[
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
            "$echo",
            new::builtin_func(
                "$echo",
                None,
                &["obj", "stderr", "newline"],
                "Echo object to stdout or stderr w/ optional newline.

                This is a low level output function that isn't intended
                for use in most code. Typically, you'll want to use
                `print` or `print_err` instead.

                TODO: Maybe implement this as a pair of instructions?

                Args:
                    obj: ObjectRef
                    stderr: Bool
                    newline: Bool

                ",
                |_, args, _| {
                    let result = check_args("$echo", &args, false, 3, Some(3));
                    if let Err(err) = result {
                        return Ok(err);
                    }

                    let obj = use_arg!(args, 0);
                    let stderr = use_arg_bool!(echo, stderr, args, 1);
                    let newline = use_arg_bool!(echo, newline, args, 2);

                    if newline {
                        if stderr {
                            eprintln!("{obj}")
                        } else {
                            println!("{obj}")
                        };
                    } else if stderr {
                        eprint!("{obj}")
                    } else {
                        print!("{obj}")
                    }

                    Ok(new::nil())
                },
            ),
        ),
    ]);

    new::builtin_module("builtins", ns, "Builtins module")
});
