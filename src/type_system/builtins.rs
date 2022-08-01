use std::sync::Arc;

use once_cell::sync::Lazy;

use super::create;

use super::bool::BOOL_TYPE;
use super::class::TYPE_TYPE;
use super::int::INT_TYPE;
use super::module::{Module, MODULE_TYPE};
use super::nil::NIL_TYPE;
use super::ns::{Namespace, NS_TYPE};
use super::str::STR_TYPE;

pub static BUILTINS: Lazy<Arc<Module>> = Lazy::new(|| {
    let mut ns = Namespace::new();
    ns.add_obj("$name", create::new_str("builtins"));
    ns.add_obj("Type", TYPE_TYPE.clone());
    ns.add_obj("Bool", BOOL_TYPE.clone());
    ns.add_obj("Int", INT_TYPE.clone());
    ns.add_obj("Module", MODULE_TYPE.clone());
    ns.add_obj("Namespace", NS_TYPE.clone());
    ns.add_obj("Nil", NIL_TYPE.clone());
    ns.add_obj("Str", STR_TYPE.clone());
    let module = Module::new("builtins", Arc::new(ns));
    Arc::new(module)
});
