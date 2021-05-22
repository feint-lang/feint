use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use super::super::class::Type;
use super::super::object::Object;
use super::BUILTIN_TYPES;

/// Built in nil type
#[derive(Debug, PartialEq)]
pub struct Nil {
    value: bool,
}

impl fmt::Display for Nil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<bool> for Nil {
    fn from(value: bool) -> Self {
        Nil { value }
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
