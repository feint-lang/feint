//! System Module
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::types::{new, Module, Namespace, ObjectTrait};
use crate::vm::{RuntimeErr, RuntimeObjResult, RuntimeResult};

use super::builtins::BUILTINS;

pub static SYSTEM: Lazy<new::obj_ref_t!(Module)> = Lazy::new(|| {
    let modules = new::map(vec![("builtins".to_owned(), BUILTINS.clone())]);

    new::builtin_module(
        "system",
        Namespace::with_entries(&[("argv", new::empty_tuple()), ("modules", modules)]),
    )
});

/// Add system module to `system.modules`, set `argv`, etc. This has to
/// be done after `SYSTEM` is created at some point during startup.
pub fn init_system_module(argv: &[String]) {
    let mut system = SYSTEM.write().unwrap();
    let ns = system.ns_mut();
    ns.set_obj("argv", new::tuple(argv.iter().map(new::str).collect()));
    let modules = ns.get_obj("modules").expect("Expected system.modules to be present");
    let modules = modules.write().unwrap();
    let modules = modules.down_to_map().expect("Expected system.modules to be a Map");
    modules.add("system".to_owned(), SYSTEM.clone());
}

/// Add a module to `system.modules`.
pub fn _add_module(name: &str, module: Module) -> RuntimeResult {
    let system = SYSTEM.read().unwrap();
    let modules = system.get_attr("modules", SYSTEM.clone())?;
    let modules = modules.write().unwrap();
    let modules = modules.down_to_map().expect("Expected system.modules to be a Map");
    let module = new::obj_ref!(module);
    modules.add(name, module);
    Ok(())
}

/// Get a module from `system.modules`.
pub fn get_module(name: &str) -> RuntimeObjResult {
    let system = SYSTEM.read().unwrap();
    let modules = system.get_attr("modules", SYSTEM.clone())?;
    let modules = modules.read().unwrap();
    let modules = modules.down_to_map().expect("Expected system.modules to be a Map");
    if let Some(module) = modules.get(name) {
        Ok(module.clone())
    } else {
        let msg = format!("module not found: {name}");
        Err(RuntimeErr::name_err(msg))
    }
}
