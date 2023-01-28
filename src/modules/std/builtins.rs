//! Builtins Module
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::types::{self, gen, new};
use crate::vm::RuntimeErr;

pub static BUILTINS: Lazy<gen::obj_ref_t!(types::module::Module)> = Lazy::new(|| {
    types::new::builtin_module(
        "std.builtins",
        "<std.builtins>",
        "std.builtins module",
        &[
            ("Type", types::class::TYPE_TYPE.clone()),
            ("Always", types::always::ALWAYS_TYPE.clone()),
            ("Bool", types::bool::BOOL_TYPE.clone()),
            ("BoundFunc", types::bound_func::BOUND_FUNC_TYPE.clone()),
            ("BuiltinFunc", types::builtin_func::BUILTIN_FUNC_TYPE.clone()),
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
                new::builtin_func(
                    "std.builtins",
                    "new_type",
                    None,
                    &["module", "name"],
                    "Make a new custom type

                    # Args
                    
                    - module: Module
                    - name: Str

                    ",
                    |_, args, _| {
                        let module = args[0].clone();
                        let name_arg = gen::use_arg!(args, 1);
                        let name = gen::use_arg_str!(new_type, name, name_arg);
                        let class = new::custom_type(module, name);
                        Ok(class)
                    },
                ),
            ),
        ],
    )
});
