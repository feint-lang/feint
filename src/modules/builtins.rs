//! Builtins Module
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::types::gen::obj_ref_t;
use crate::types::module::Module;
use crate::types::new;
use crate::types::ns::Namespace;

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

pub static BUILTINS: Lazy<obj_ref_t!(Module)> = Lazy::new(|| {
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
    ]);

    new::builtin_module("builtins", "builtins.fi", ns, "Builtins module")
});
