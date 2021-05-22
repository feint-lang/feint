use std::collections::HashMap;
use std::sync::Arc;

use lazy_static::lazy_static;

use super::super::{Object, Type};

lazy_static! {
    pub static ref BUILTIN_TYPES: HashMap<&'static str, Arc<Type>> = {
        let mut builtins = HashMap::new();
        let mut insert = |n| builtins.insert(n, Arc::new(Type::new("builtins", n)));
        insert("Nil");
        insert("Bool");
        insert("Float");
        insert("Int");
        builtins
    };
}
