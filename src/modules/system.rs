//! System Module
use std::path::Path;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::config::CONFIG;
use crate::exe::Executor;
use crate::types::gen::{obj_ref, obj_ref_t};
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
                ("proc".to_owned(), PROC.clone()),
            ]),
        ),
    ];

    new::builtin_module("system", Namespace::with_entries(&entries), "System module")
});

/// Add a module to `system.modules`.
pub fn _add_module(name: &str, module: Module) -> ObjectRef {
    let system = SYSTEM.read().unwrap();
    let modules = system.get_attr("modules", SYSTEM.clone());
    let modules = modules.write().expect("Expected system module to be an object");
    let modules = modules.down_to_map().expect("Expected system.modules to be a Map");
    let module = obj_ref!(module);
    modules.add(name, module.clone());
    module
}

/// Get a module from `system.modules`.
pub fn get_module(name: &str) -> Result<ObjectRef, RuntimeErr> {
    let system = SYSTEM.read().unwrap();
    let modules_ref = system.get_attr("modules", SYSTEM.clone());

    let modules_guard = modules_ref.read().unwrap();
    let modules =
        modules_guard.down_to_map().expect("Expected system.modules to be a Map");

    if let Some(module) = modules.get(name) {
        Ok(module.clone())
    } else {
        drop(modules_guard);

        let obj_ref = load_fi_module(name)?;
        let obj = obj_ref.read().unwrap();

        if obj.is_mod() {
            let modules_guard = modules_ref.write().unwrap();
            let modules = modules_guard
                .down_to_map()
                .expect("Expected system.modules to be a Map");
            modules.add(name, obj_ref.clone());
        }

        Ok(obj_ref.clone())
    }
}

pub fn load_fi_module(name: &str) -> Result<ObjectRef, RuntimeErr> {
    let config = CONFIG.read().unwrap();
    let search_path = config.get_str("builtin_module_search_path")?;

    let mut path = Path::new(search_path).to_path_buf();
    for segment in name.split('.') {
        path = path.join(segment);
    }
    path.set_extension("fi");

    drop(config);

    if path.is_file() {
        let mut executor = Executor::for_add_module();
        let result = executor.load_module(name, path.as_path());
        return match result {
            Ok(module) => Ok(obj_ref!(module)),
            Err(err) => {
                if let Some(code) = err.exit_code() {
                    Err(RuntimeErr::exit(code))
                } else {
                    let msg = format!("{name}:\n\n{err}");
                    Ok(new::module_could_not_be_loaded(msg, SYSTEM.clone()))
                }
            }
        };
    }

    Ok(new::module_not_found_err(name, SYSTEM.clone()))
}
