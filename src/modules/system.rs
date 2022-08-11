//! System Module
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::types::{create, Module, Namespace};

use super::BUILTINS;
use super::FILE;

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
