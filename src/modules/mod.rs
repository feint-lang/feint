use ::std::sync::{Arc, RwLock};
use once_cell::sync::Lazy;

use crate::types::gen::{obj_ref, obj_ref_t};
use crate::types::{Map, ObjectRef, ObjectTrait};

pub mod std;

/// This mirrors `system.modules`. It provides a way to access
/// modules in Rust code (e.g., in the VM).
static MODULES: Lazy<obj_ref_t!(Map)> = Lazy::new(|| obj_ref!(Map::default()));

/// Add module to `std.system.modules`.
pub fn add_module(name: &str, module: ObjectRef) {
    let modules = MODULES.write().unwrap();
    let modules = modules.down_to_map().unwrap();
    modules.add(name, module);
}

/// Get module from `system.modules`.
///
/// XXX: Panics if the module doesn't exist (since that shouldn't be
///      possible).
pub fn get_module(name: &str) -> ObjectRef {
    let modules = MODULES.read().unwrap();
    let modules = modules.down_to_map().unwrap();
    if let Some(module) = modules.get(name) {
        module.clone()
    } else {
        panic!("Module not registered: {name}");
    }
}

/// Get module from `system.modules`.
///
/// XXX: This will return `None` if the module doesn't exist. Generally,
///      this should only be used during bootstrap. In most cases,
///      `get_module` should be used instead.
pub fn maybe_get_module(name: &str) -> Option<ObjectRef> {
    let modules = MODULES.read().unwrap();
    let modules = modules.down_to_map().unwrap();
    modules.get(name)
}
