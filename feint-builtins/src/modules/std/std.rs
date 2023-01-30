//! Root of the std module hierarchy containing builtins/prelude.
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use feint_code_gen::{obj_ref_t, use_arg, use_arg_str};

use crate::types::{self, new};

pub static STD: Lazy<obj_ref_t!(types::module::Module)> = Lazy::new(|| {
    new::intrinsic_module(
        "std",
        "<std>",
        "std module (builtins)",
        &[
            ("Type", types::class::TYPE_TYPE.clone()),
            ("Always", types::always::ALWAYS_TYPE.clone()),
            ("Bool", types::bool::BOOL_TYPE.clone()),
            ("BoundFunc", types::bound_func::BOUND_FUNC_TYPE.clone()),
            ("IntrinsicFunc", types::intrinsic_func::INTRINSIC_FUNC_TYPE.clone()),
            ("Closure", types::closure::CLOSURE_TYPE.clone()),
            ("Err", types::err::ERR_TYPE.clone()),
            ("ErrType", types::err_type::ERR_TYPE_TYPE.clone()),
            ("File", types::file::FILE_TYPE.clone()),
            ("Func", types::func::FUNC_TYPE.clone()),
            ("Float", types::float::FLOAT_TYPE.clone()),
            ("Int", types::int::INT_TYPE.clone()),
            ("Iterator", types::iterator::ITERATOR_TYPE.clone()),
            ("List", types::list::LIST_TYPE.clone()),
            ("Map", types::map::MAP_TYPE.clone()),
            ("Module", types::module::MODULE_TYPE.clone()),
            ("Nil", types::nil::NIL_TYPE.clone()),
            ("Str", types::str::STR_TYPE.clone()),
            ("Tuple", types::tuple::TUPLE_TYPE.clone()),
            (
                "new_type",
                new::intrinsic_func(
                    "std",
                    "new_type",
                    None,
                    &["module", "name"],
                    "Make a new custom type

                    # Args
                    
                    - module: Module
                    - name: Str

                    ",
                    |_, args| {
                        let module = args[0].clone();
                        let name_arg = use_arg!(args, 1);
                        let name = use_arg_str!(new_type, name, name_arg);
                        let class = new::custom_type(module, name);
                        class
                    },
                ),
            ),
        ],
    )
});
