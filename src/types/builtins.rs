use std::collections::HashMap;

use lazy_static::lazy_static;

use super::Type;

lazy_static! {
    pub static ref BUILTIN_TYPES: HashMap<&'static str, Type> = {
        let mut types = HashMap::new();
        types.insert("None", Type::new("builtins", "None", vec![]));
        types.insert("Bool", Type::new("builtins", "Bool", vec!["value"]));
        types.insert("Float", Type::new("builtins", "Float", vec!["value"]));
        types.insert("Int", Type::new("builtins", "Int", vec!["value"]));
        types
    };
}
