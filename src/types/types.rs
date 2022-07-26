use std::collections::HashMap;

use lazy_static::lazy_static;

use super::class::Type;

lazy_static! {
    pub static ref TYPES: HashMap<&'static str, Type> = [
        ("Type", Type::new("builtins", "Type")),
        ("Nil", Type::new("builtins", "Nil")),
        ("Bool", Type::new("builtins", "Bool")),
        ("Int", Type::new("builtins", "Int")),
        ("Float", Type::new("builtins", "Float")),
        ("Func", Type::new("builtins", "Func")),
        ("NativeFunc", Type::new("builtins", "NativeFunc")),
        ("Str", Type::new("builtins", "Str")),
        ("Tuple", Type::new("builtins", "Tuple")),
    ]
    .iter()
    .cloned()
    .collect();
}
