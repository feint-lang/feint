//! Root of the std module hierarchy containing builtins/prelude.
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use feint_code_gen::{obj_ref_t, use_arg, use_arg_str};

use crate::{types, BUILTINS};

pub static STD: Lazy<obj_ref_t!(types::module::Module)> = Lazy::new(|| {
    BUILTINS.module(
        "std",
        "<std>",
        "std module (builtins)",
        &[
            ("Type", BUILTINS.type_type()),
            ("Always", BUILTINS.always_type()),
            ("Bool", BUILTINS.bool_type()),
            ("BoundFunc", BUILTINS.bound_func_type()),
            ("IntrinsicFunc", BUILTINS.intrinsic_func_type()),
            ("Closure", BUILTINS.closure_type()),
            ("Err", BUILTINS.err_type()),
            ("ErrType", BUILTINS.err_type_type()),
            ("File", BUILTINS.file_type()),
            ("Func", BUILTINS.func_type()),
            ("Float", BUILTINS.float_type()),
            ("Int", BUILTINS.int_type()),
            ("Iterator", BUILTINS.iterator_type()),
            ("List", BUILTINS.list_type()),
            ("Map", BUILTINS.map_type()),
            ("Module", BUILTINS.module_type()),
            ("Nil", BUILTINS.nil_type()),
            ("Str", BUILTINS.str_type()),
            ("Tuple", BUILTINS.tuple_type()),
            // (
            //     "new_type",
            //     BUILTINS.intrinsic_func(
            //         "std",
            //         "new_type",
            //         None,
            //         &["module", "name"],
            //         "Make a new custom type
            //
            //         # Args
            //
            //         - module: Module
            //         - name: Str
            //
            //         ",
            //         |_, args| {
            //             let module = args[0].clone();
            //             let name_arg = use_arg!(args, 1);
            //             let name = use_arg_str!(new_type, name, name_arg);
            //             let class = BUILTINS.custom_type(module, name);
            //             class
            //         },
            //     ),
            // ),
        ],
    )
});
