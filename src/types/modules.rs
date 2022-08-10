//! Builtins Module
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::builtin_funcs::get_builtin_func_specs;

use super::create;

use super::bool::BOOL_TYPE;
use super::bound_func::BOUND_FUNC_TYPE;
use super::builtin_func::BUILTIN_FUNC_TYPE;
use super::class::TYPE_TYPE;
use super::closure::CLOSURE_TYPE;
use super::float::FLOAT_TYPE;
use super::func::FUNC_TYPE;
use super::int::INT_TYPE;
use super::list::LIST_TYPE;
use super::map::MAP_TYPE;
use super::module::{Module, MODULE_TYPE};
use super::nil::NIL_TYPE;
use super::ns::Namespace;
use super::str::STR_TYPE;
use super::tuple::TUPLE_TYPE;

pub static BUILTINS: Lazy<Arc<RwLock<Module>>> = Lazy::new(|| {
    let mut entries = vec![
        ("$name", create::new_str("builtins")),
        ("Type", TYPE_TYPE.clone()),
        ("Bool", BOOL_TYPE.clone()),
        ("BoundFunc", BOUND_FUNC_TYPE.clone()),
        ("BuiltinFunc", BUILTIN_FUNC_TYPE.clone()),
        ("Closure", CLOSURE_TYPE.clone()),
        ("Func", FUNC_TYPE.clone()),
        ("Float", FLOAT_TYPE.clone()),
        ("Int", INT_TYPE.clone()),
        ("List", LIST_TYPE.clone()),
        ("Map", MAP_TYPE.clone()),
        ("Module", MODULE_TYPE.clone()),
        ("Nil", NIL_TYPE.clone()),
        ("Str", STR_TYPE.clone()),
        ("Tuple", TUPLE_TYPE.clone()),
    ];

    for spec in get_builtin_func_specs() {
        let (name, params, func) = spec;
        entries.push((name, create::new_builtin_func(name, params, func)));
    }

    create::new_builtin_module("builtins", Namespace::with_entries(&entries))
});

pub static SYSTEM: Lazy<Arc<RwLock<Module>>> = Lazy::new(|| {
    let modules = create::new_map(vec![("builtins".to_string(), BUILTINS.clone())]);

    create::new_builtin_module(
        "system",
        Namespace::with_entries(&[
            ("$name", create::new_str("system")),
            ("modules", modules),
        ]),
    )
});
