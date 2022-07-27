use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;

use super::class::{Type, TypeRef};

lazy_static! {
    /// Builtin types are defined statically for bootstrapping.
    pub static ref BUILTIN_TYPES: HashMap<&'static str, TypeRef> = [
        ("Bool", Arc::new(Mutex::new(Type::new("builtins", "Bool")))),
        ("BuiltinFunc", Arc::new(Mutex::new(Type::new("builtins", "BuiltinFunc")))),
        ("Float", Arc::new(Mutex::new(Type::new("builtins", "Float")))),
        ("Func", Arc::new(Mutex::new(Type::new("builtins", "Func")))),
        ("Int", Arc::new(Mutex::new(Type::new("builtins", "Int")))),
        ("Namespace", Arc::new(Mutex::new(Type::new("builtins", "Namespace")))),
        ("Nil", Arc::new(Mutex::new(Type::new("builtins", "Nil")))),
        ("Str", Arc::new(Mutex::new(Type::new("builtins", "Str")))),
        ("Tuple", Arc::new(Mutex::new(Type::new("builtins", "Tuple")))),
        ("Type", Arc::new(Mutex::new(Type::new("builtins", "Type")))),
    ]
    .iter()
    .cloned()
    .collect();
}
