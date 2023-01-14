//! System Module
use std::path::Path;
use std::sync::{Arc, RwLock};

use num_bigint::BigInt;
use num_traits::ToPrimitive;
use once_cell::sync::Lazy;

use crate::config::CONFIG;
use crate::exe::Executor;
use crate::types::{new, Module, Namespace, ObjectRef, ObjectTrait};
use crate::util::check_args;
use crate::vm::RuntimeErr;

use super::builtins::BUILTINS;
use super::proc::PROC;

pub static SYSTEM: Lazy<new::obj_ref_t!(Module)> = Lazy::new(|| {
    let entries: Vec<(&str, ObjectRef)> = vec![
        ("argv", new::empty_tuple()),
        (
            "modules",
            new::map(vec![
                ("builtins".to_owned(), BUILTINS.clone()),
                ("proc".to_owned(), PROC.clone()),
            ]),
        ),
        (
            "exit",
            new::builtin_func(
                "exit",
                None,
                &[""],
                "Exit program with return code.

                Args:
                    return_code?: Int = 0",
                |_, args, _| {
                    let name = "system.exit()";

                    let result = check_args(name, &args, true, 0, Some(1));
                    if let Err(err) = result {
                        return Ok(err);
                    }
                    let (n_args, _, var_args_obj) = result.unwrap();

                    if n_args == 0 {
                        return Err(RuntimeErr::exit(0));
                    }

                    let var_args = var_args_obj.read().unwrap();
                    let code_arg = var_args.get_item(0, var_args_obj.clone());
                    let code = code_arg.read().unwrap();

                    if let Some(int) = code.get_int_val() {
                        let max = u8::MAX;
                        if int < &BigInt::from(0) || int > &BigInt::from(max) {
                            let message =
                                format!("{name} return code must be in [0, {max}]");
                            Ok(new::arg_err(message, new::nil()))
                        } else {
                            let code = int.to_u8().unwrap();
                            Err(RuntimeErr::exit(code))
                        }
                    } else {
                        let message =
                            format!("{name} expected an Int; got {:?}", &*code);
                        Ok(new::arg_err(message, new::nil()))
                    }
                },
            ),
        ),
    ];

    new::builtin_module("system", Namespace::with_entries(&entries), "System module")
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
pub fn add_module(name: &str, module: Module) -> ObjectRef {
    let system = SYSTEM.read().unwrap();
    let modules = system.get_attr("modules", SYSTEM.clone());
    let modules = modules.write().expect("Expected system module to be an object");
    let modules = modules.down_to_map().expect("Expected system.modules to be a Map");
    let module = new::obj_ref!(module);
    modules.add(name, module.clone());
    module
}

/// Get a module from `system.modules`.
pub fn get_module(name: &str) -> Result<(ObjectRef, bool), RuntimeErr> {
    let system = SYSTEM.read().unwrap();
    let modules_ref = system.get_attr("modules", SYSTEM.clone());
    let modules_guard =
        modules_ref.read().expect("Expected system module to be an object");
    let modules =
        modules_guard.down_to_map().expect("Expected system.modules to be a Map");
    if let Some(module) = modules.get(name) {
        Ok((module.clone(), false))
    } else {
        // XXX: Prevent deadlock when calling add_module.
        drop(modules_guard);

        let config = CONFIG.read().unwrap();
        let search_path = config.get_str("builtin_module_search_path")?;

        let file_name = format!("{search_path}/{name}.fi");
        let path = Path::new(file_name.as_str());

        drop(config);

        let module = if path.is_file() {
            let mut executor = Executor::for_add_module();
            let result = executor.load_module(name, path);
            match result {
                Ok(module) => add_module(name, module),
                Err(err) => {
                    let msg = format!("{name}:\n\n{err}");
                    new::module_could_not_be_loaded(msg, SYSTEM.clone())
                }
            }
        } else {
            new::module_not_found_err(name, SYSTEM.clone())
        };

        Ok((module, true))
    }
}
