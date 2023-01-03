//! System Module
use std::sync::{Arc, RwLock};

use num_bigint::BigInt;
use num_traits::ToPrimitive;
use once_cell::sync::Lazy;

use crate::types::{new, Module, Namespace, ObjectRef, ObjectTrait};
use crate::util::check_args;
use crate::vm::{RuntimeErr, RuntimeObjResult, RuntimeResult};

use super::builtins::BUILTINS;

pub static SYSTEM: Lazy<new::obj_ref_t!(Module)> = Lazy::new(|| {
    let entries: Vec<(&str, ObjectRef)> = vec![
        ("argv", new::empty_tuple()),
        ("modules", new::map(vec![("builtins".to_owned(), BUILTINS.clone())])),
        (
            // Exit program with return code.
            //
            // Args:
            //     return_code?: Int = 0
            "exit",
            new::builtin_func("exit", None, &[""], |_, args, _| {
                let name = "system.exit()";
                let (n_args, _, var_args) = check_args(name, 1, Some(1), true, &args)?;

                if n_args == 0 {
                    return Err(RuntimeErr::exit(0));
                }

                let var_args = var_args.read().unwrap();
                let code_arg = var_args.get_item(0)?;
                let code = code_arg.read().unwrap();

                if let Some(int) = code.get_int_val() {
                    let max = u8::MAX;
                    if int < &BigInt::from(0) || int > &BigInt::from(max) {
                        let message =
                            format!("{name} return code must be in [0, {max}]");
                        Err(RuntimeErr::arg_err(message))
                    } else {
                        let code = int.to_u8().unwrap();
                        Err(RuntimeErr::exit(code))
                    }
                } else {
                    let message = format!("{name} expected an Int; got {:?}", &*code);
                    Err(RuntimeErr::arg_err(message))
                }
            }),
        ),
    ];

    new::builtin_module("system", Namespace::with_entries(&entries))
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
