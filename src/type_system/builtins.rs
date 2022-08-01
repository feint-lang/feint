use std::sync::Arc;

use once_cell::sync::Lazy;

use super::base::ObjectRef;
use super::bool::{Bool, BoolType, BOOL_TYPE};
use super::class::{Type, TypeType, TYPE_TYPE};
use super::int::{Int, IntType, INT_TYPE};
use super::module::{Module, ModuleType, MODULE_TYPE};
use super::nil::{Nil, NilType, NIL_TYPE};
use super::ns::{Namespace, NamespaceType, NS_TYPE};
use super::str::{Str, StrType, STR_TYPE};

use super::create;

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
