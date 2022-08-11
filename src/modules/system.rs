//! System Module
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::types::{create, Module, Namespace, ObjectTrait};
use crate::vm::{Code, RuntimeErr, RuntimeObjResult};

use super::builtins::BUILTINS;
use super::file::FILE;

pub static SYSTEM: Lazy<Arc<RwLock<Module>>> = Lazy::new(|| {
    let modules = create::new_map(vec![
        ("builtins".to_string(), BUILTINS.clone()),
        ("file".to_string(), FILE.clone()),
    ]);

    create::new_builtin_module(
        "system",
        Namespace::with_entries(&[
            ("$name", create::new_str("system")),
            ("modules", modules),
        ]),
    )
});

/// Add a module to system.modules.
pub fn add_module(name: &str, code: Code) -> RuntimeObjResult {
    let system = SYSTEM.read().unwrap();
    let modules = system.get_attr("modules")?;
    let modules = modules.write().unwrap();
    let modules = modules.down_to_map().expect("Expected system.modules to be a Map");
    let module = create::new_module(name, Namespace::new(), code);
    modules.add(name, module.clone());
    Ok(module)
}

/// Get a module from system.modules.
pub fn get_module(name: &str) -> RuntimeObjResult {
    let system = SYSTEM.read().unwrap();
    let modules = system.get_attr("modules")?;
    let modules = modules.read().unwrap();
    let modules = modules.down_to_map().expect("Expected system.modules to be a Map");
    if let Some(module) = modules.get(name) {
        Ok(module.clone())
    } else {
        let msg = format!("module not found: {name}");
        Err(RuntimeErr::new_name_err(msg))
    }
}