//! System Module
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::types::gen::obj_ref_t;
use crate::types::{new, Module, ObjectRef, ObjectTrait};
use crate::vm::RuntimeErr;

pub static SYSTEM: Lazy<obj_ref_t!(Module)> = Lazy::new(|| {
    let modules = new::map(vec![
        ("std".to_owned(), new::builtin_stub_module("std")),
        ("std.args".to_owned(), new::builtin_stub_module("std.args")),
        ("std.builtins".to_owned(), super::builtins::BUILTINS.clone()),
        ("std.proc".to_owned(), super::proc::PROC.clone()),
        ("std.system".to_owned(), new::nil()),
        ("std.test".to_owned(), new::builtin_stub_module("std.test")),
    ]);

    new::builtin_module(
        "std.system",
        "std.system",
        "std.system module",
        &[("argv", new::empty_tuple()), ("modules", modules)],
    )
});

/// Get a module from `system.modules`.
pub fn get_module(name: &str) -> Result<ObjectRef, RuntimeErr> {
    let system = SYSTEM.read().unwrap();
    let modules = system.get_attr("modules", SYSTEM.clone());
    let modules = modules.read().unwrap();
    if let Some(modules) = modules.down_to_map() {
        if let Some(module) = modules.get(name) {
            Ok(module.clone())
        } else {
            Ok(new::module_not_found_err(name, SYSTEM.clone()))
        }
    } else {
        Err(RuntimeErr::type_err("Expected system.modules to be a Map; got {modules}"))
    }
}
