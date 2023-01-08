//use std::process::Command;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::types::{new, Args, Module, Namespace, ObjectRef};

pub static PROC: Lazy<new::obj_ref_t!(Module)> = Lazy::new(|| {
    let entries: Vec<(&str, ObjectRef)> = vec![
        // TODO:
    ];

    new::builtin_module("proc", Namespace::with_entries(&entries), "Proc")
});
