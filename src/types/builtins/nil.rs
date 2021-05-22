use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use lazy_static::lazy_static;

use super::super::class::Type;
use super::super::object::Object;
use super::BUILTIN_TYPES;

lazy_static! {
    pub static ref NIL: Nil = Nil {};
}

/// Built in nil type
#[derive(Debug, PartialEq)]
pub struct Nil {}

impl fmt::Display for Nil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "nil")
    }
}

impl Object for Nil {
    fn class(&self) -> Arc<Type> {
        BUILTIN_TYPES.get("Nil").unwrap().clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
