//! Builtins Module
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::types;

pub static BUILTINS: Lazy<types::gen::obj_ref_t!(types::module::Module)> =
    Lazy::new(|| {
        types::new::builtin_module(
            "std.builtins",
            "std.builtins",
            types::ns::Namespace::with_entries(&[
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
                ("List", types::list::LIST_TYPE.clone()),
                ("Map", types::map::MAP_TYPE.clone()),
                ("Module", types::module::MODULE_TYPE.clone()),
                ("Nil", types::nil::NIL_TYPE.clone()),
                ("Str", types::str::STR_TYPE.clone()),
                ("Tuple", types::tuple::TUPLE_TYPE.clone()),
            ]),
            "Builtins module",
        )
    });
