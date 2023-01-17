//! System Module
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::types::gen::obj_ref_t;
use crate::types::{new, Module, Namespace, ObjectRef, ObjectTrait};
use crate::vm::RuntimeErr;

use super::builtins::BUILTINS;
use super::proc::PROC;

pub static SYSTEM: Lazy<obj_ref_t!(Module)> = Lazy::new(|| {
    let entries: Vec<(&str, ObjectRef)> = vec![
        ("argv", new::empty_tuple()),
        (
            "modules",
            new::map(vec![
                ("builtins".to_owned(), BUILTINS.clone()),
                ("system".to_owned(), new::nil()),
                ("proc".to_owned(), PROC.clone()),
            ]),
        ),
    ];

    new::builtin_module(
        "system",
        "system.fi",
        Namespace::with_entries(&entries),
        "System module",
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
