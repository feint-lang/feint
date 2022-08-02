//! Builtins Module
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::builtin_funcs::get_builtin_func_specs;

use super::create;

use super::bool::BOOL_TYPE;
use super::builtin_func::BUILTIN_FUNC_TYPE;
use super::class::TYPE_TYPE;
use super::float::FLOAT_TYPE;
use super::func::FUNC_TYPE;
use super::int::INT_TYPE;
use super::module::{Module, MODULE_TYPE};
use super::nil::NIL_TYPE;
use super::ns::Namespace;
use super::str::STR_TYPE;
use super::tuple::TUPLE_TYPE;

pub static BUILTINS: Lazy<Arc<RwLock<Module>>> = Lazy::new(|| {
    let mut ns = Namespace::new();

    ns.add_obj("$name", create::new_str("builtins"));
    ns.add_obj("Type", TYPE_TYPE.clone());
    ns.add_obj("Bool", BOOL_TYPE.clone());
    ns.add_obj("BuiltinFunc", BUILTIN_FUNC_TYPE.clone());
    ns.add_obj("Func", FUNC_TYPE.clone());
    ns.add_obj("Float", FLOAT_TYPE.clone());
    ns.add_obj("Int", INT_TYPE.clone());
    ns.add_obj("Module", MODULE_TYPE.clone());
    ns.add_obj("Nil", NIL_TYPE.clone());
    ns.add_obj("Str", STR_TYPE.clone());
    ns.add_obj("Tuple", TUPLE_TYPE.clone());

    for spec in get_builtin_func_specs() {
        let (name, params, func) = spec;
        let func = create::new_builtin_func(name, params, func);
        ns.add_obj(name, func);
    }

    create::new_module("builtins", ns)
});
