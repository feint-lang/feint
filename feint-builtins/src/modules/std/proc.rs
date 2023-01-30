//use std::process::Command;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use feint_code_gen::obj_ref_t;

use crate::types::{new, Module};

pub static PROC: Lazy<obj_ref_t!(Module)> = Lazy::new(|| {
    new::intrinsic_module(
        "std.proc",
        "<std.proc>",
        "Proc module",
        &[
            // TODO:
        ],
    )
});
