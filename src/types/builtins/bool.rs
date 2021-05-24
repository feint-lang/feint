use std::fmt;

use builtin_object_derive::BuiltinObject;

/// Built in boolean type
#[derive(Debug, PartialEq, BuiltinObject)]
pub struct Bool {
    class: Rc<Type>,
    value: bool,
}

impl Bool {
    pub fn new(class: Rc<Type>, value: bool) -> Self {
        Self { class: class.clone(), value }
    }
}

impl fmt::Display for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
