use crate::types::{new, ObjectRef, ObjectTrait};

use super::builtins::BUILTINS;
use super::system::{load_fi_module, SYSTEM};

/// Bootstrap modules.
///
/// 1. Set `system.argv`.
/// 2. Add `system` module to `system.modules`. This has to be done
///    after `SYSTEM` is created at some point during startup to avoid
///    a deadlock.
/// 3. Add objects implemented in FeInt modules to base modules
///    implemented in Rust.
pub fn bootstrap(argv: &[String]) {
    {
        // XXX: New scopes ensure all locks are dropped.
        let mut system = SYSTEM.write().unwrap();

        let argv = new::tuple(argv.iter().map(new::str).collect());
        system.ns_mut().add_obj("argv", argv);

        {
            let modules = system.get_attr("modules", SYSTEM.clone());
            let modules = modules.write().unwrap();
            if let Some(modules) = modules.down_to_map() {
                modules.add("system".to_owned(), SYSTEM.clone());
            } else {
                panic!("Expected system.modules to be a Map; got {modules}");
            }
        }
    }

    extend_module("builtins", BUILTINS.clone());
    extend_module("system", SYSTEM.clone());
}

/// Extend a module implemented in Rust with the corresponding module
/// implemented in FeInt.
///
/// XXX: Another way to approach this would be to import objects from
///      Rust-implemented modules into FeInt-implemented modules and
///      then re-export them. To do this, the Rust-implemented modules
///      would need to be named to make them private (e.g.,
///      `$builtins`), but currently it's not possible to import a name
///      starting with a `$`.
fn extend_module(name: &str, base_module_ref: ObjectRef) {
    let result = load_fi_module(name);

    match result {
        Ok(module_ref) => {
            let mut base_module = base_module_ref.write().unwrap();
            let module = module_ref.read().unwrap();
            if let Some(module) = module.down_to_mod() {
                for (name, val) in module.iter_globals() {
                    base_module.ns_mut().add_obj(name, val.clone());
                }
            } else {
                panic!("Expected {name}.fi to be a module; got {module}");
            }
        }
        Err(err) => {
            panic!("Could not load {name}.fi module: {err}");
        }
    }
}
