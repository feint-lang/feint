use ::std::borrow::Cow;
use ::std::collections::HashMap;
use ::std::io::Read;
use ::std::path::Path;
use ::std::sync::{Arc, RwLock};

use flate2::read::GzDecoder;
use once_cell::sync::Lazy;
use tar::Archive as TarArchive;

use feint_code_gen::{obj_ref, obj_ref_t};

use crate::types::map::Map;
use crate::types::{ObjectRef, ObjectTrait};

pub mod std;
pub use self::std::STD;

/// This mirrors `system.modules`. It provides a way to access
/// modules in Rust code (e.g., in the VM).
pub static MODULES: Lazy<obj_ref_t!(Map)> = Lazy::new(|| obj_ref!(Map::default()));

/// Add module to `std.system.modules`.
pub fn add_module(name: &str, module: ObjectRef) {
    let modules = MODULES.write().unwrap();
    let modules = modules.down_to_map().unwrap();
    modules.insert(name, module);
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

/// At build time, a compressed archive is created containing the
/// std .fi module files (see `build.rs`).
///
/// At runtime, the module file data is read out and stored in a map
/// (lazily). When a std module is imported, the file data is read from
/// this map rather than reading from disk.
///
/// The utility of this is that we don't need an install process that
/// copies the std module files into some location on the file system
/// based on the location of the current executable or anything like
/// that.
pub static STD_FI_MODULES: Lazy<HashMap<String, Vec<u8>>> = Lazy::new(|| {
    let archive_bytes: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/modules.tgz"));
    let decoder = GzDecoder::new(archive_bytes);
    let mut archive = TarArchive::new(decoder);
    let mut modules = HashMap::new();
    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let path: Cow<'_, Path> = entry.path().unwrap();
        let path = path.to_str().unwrap().to_owned();
        let mut result = Vec::new();
        entry.read_to_end(&mut result).unwrap();
        modules.insert(path, result);
    }
    modules
});
