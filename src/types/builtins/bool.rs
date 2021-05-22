use std::fmt;

use lazy_static::lazy_static;

use builtin_object_derive::BuiltinObject;

lazy_static! {
    pub static ref TRUE: Bool = Bool::new(true);
    pub static ref FALSE: Bool = Bool::new(false);
}

/// Built in boolean type
#[derive(Debug, PartialEq, BuiltinObject)]
pub struct Bool {
    value: bool,
}

impl Bool {
    pub fn new(value: bool) -> Self {
        let instance = Self { value };
        instance.class().clone();
        instance
    }
}

impl From<bool> for Bool {
    fn from(value: bool) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
