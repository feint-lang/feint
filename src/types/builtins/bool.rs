use std::fmt;

use lazy_static::lazy_static;

use builtin_object_derive::BuiltinObject;

lazy_static! {
    pub static ref TRUE: Bool = Bool { value: true };
    pub static ref FALSE: Bool = Bool { value: false };
}

/// Built in boolean type
#[derive(Debug, PartialEq, BuiltinObject)]
pub struct Bool {
    value: bool,
}

impl From<bool> for Bool {
    fn from(value: bool) -> Self {
        Bool { value }
    }
}

impl fmt::Display for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
