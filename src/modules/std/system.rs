//! System Module
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::types::gen::obj_ref_t;
use crate::types::{new, Module, ObjectRef, ObjectTrait};

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
pub fn get_module(name: &str) -> ObjectRef {
    let system = SYSTEM.read().unwrap();
    let modules = system.get_attr("modules", SYSTEM.clone());
    let modules = modules.read().unwrap();
    let modules = modules.down_to_map().unwrap();
    if let Some(module) = modules.get(name) {
        module.clone()
    } else {
        new::module_not_found_err(name, SYSTEM.clone())
    }
}
