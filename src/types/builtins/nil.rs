use std::fmt;

use lazy_static::lazy_static;

use builtin_object_derive::BuiltinObject;

lazy_static! {
    pub static ref NIL: Nil = Nil {};
}

/// Built in nil type
#[derive(Debug, PartialEq, BuiltinObject)]
pub struct Nil {}

impl fmt::Display for Nil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "nil")
    }
}
