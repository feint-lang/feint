//! Built in nil type

use std::fmt;

use lazy_static::lazy_static;

use builtin_object_derive::BuiltinObject;

lazy_static! {
    pub static ref NIL: Nil = Nil::new();
}

/// Built in nil type
#[derive(Debug, PartialEq, BuiltinObject)]
pub struct Nil;

impl Nil {
    pub fn new() -> Self {
        let instance = Self {};
        instance.class().clone();
        instance
    }
}

impl fmt::Display for Nil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "nil")
    }
}
