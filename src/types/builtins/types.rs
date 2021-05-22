use std::collections::HashMap;
use std::sync::Arc;

use lazy_static::lazy_static;

use super::super::{Object, Type};
use super::{Bool, Nil};

lazy_static! {
    pub static ref BUILTIN_TYPES: Builtins = {
        let mut builtins = Builtins::new();
        builtins.add("Bool");
        builtins.add("Float");
        builtins.add("Int");
        builtins.add("Nil");
        builtins
    };
}

pub struct Builtins(HashMap<&'static str, Arc<Type>>);

impl Builtins {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn add(&mut self, name: &'static str) {
        let class = Arc::new(Type::new("builtins", name));
        self.0.insert(name, class);
    }

    /// Get builtin type by name. Panic if a type doesn't exist with the
    /// specified name.
    pub fn get(&self, name: &str) -> &Arc<Type> {
        let message = format!("Unknown builtin type: {}", name);
        let class = self.0.get(name).expect(message.as_str());
        class
    }
}
