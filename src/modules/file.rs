//! File Module
use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader};
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::RuntimeErr;

use crate::types::{create, Args, Module, Namespace};

pub static FILE: Lazy<Arc<RwLock<Module>>> = Lazy::new(|| {
    create::new_builtin_module(
        "file",
        Namespace::with_entries(&[
            ("$name", create::new_str("file")),
            (
                "read",
                create::new_builtin_func("read", &["file_name"], |_, args: Args, _| {
                    let arg = args.get(0).unwrap();
                    let arg = arg.read().unwrap();
                    if let Some(file_name) = arg.get_str_val() {
                        read_to_string(file_name).map(create::new_str).map_err(|err| {
                            RuntimeErr::could_not_read_file(err.to_string())
                        })
                    } else {
                        Err(RuntimeErr::arg_err("file_name: expected string"))
                    }
                }),
            ),
            (
                "read_lines",
                create::new_builtin_func(
                    "read_lines",
                    &["file_name"],
                    |_, args: Args, _| {
                        let arg = args.get(0).unwrap();
                        let arg = arg.read().unwrap();
                        if let Some(file_name) = arg.get_str_val() {
                            let file = File::open(file_name);
                            file.map(|file| {
                                let reader = BufReader::new(file);
                                let lines = reader
                                    .lines()
                                    // TODO: Handle lines that can't be read
                                    .map(|line| create::new_str(line.unwrap()))
                                    .collect();
                                create::new_tuple(lines)
                            })
                            .map_err(|err| {
                                RuntimeErr::could_not_read_file(err.to_string())
                            })
                        } else {
                            Err(RuntimeErr::arg_err("file_name: expected string"))
                        }
                    },
                ),
            ),
        ]),
    )
});
