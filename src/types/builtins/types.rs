use std::collections::HashMap;
use std::sync::Arc;

use lazy_static::lazy_static;

use super::super::{Object, Type};
use super::{Bool, Nil};

lazy_static! {
    pub static ref BUILTINS: Builtins = {
        let mut builtins = Builtins::new();
        builtins.add("Bool");
        builtins.add("Float");
        builtins.add("Int");
        builtins.add("Nil");
        builtins
    };
}

pub struct Builtins {
    types: HashMap<&'static str, Arc<Type>>,
    pub TRUE: Bool,
    pub FALSE: Bool,
    pub NIL: Nil,
}

impl Builtins {
    fn new() -> Self {
        Self {
            types: HashMap::new(),
            TRUE: Bool::from(true),
            FALSE: Bool::from(false),
            NIL: Nil {},
        }
    }

    fn add(&mut self, name: &'static str) {
        let class = Arc::new(Type::new("builtins", name));
        self.types.insert(name, class);
    }

    /// Get builtin type by name. Panic if a type doesn't exist with the
    /// specified name. Note that this will bump the ref count on the
    /// type, so the caller shouldn't do that (XXX).
    pub fn get(&self, name: &str) -> Arc<Type> {
        let message = format!("Unknown builtin type: {}", name);
        let class = self.types.get(name).expect(message.as_str());
        class.clone()
    }
}
