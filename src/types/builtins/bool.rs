use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use super::super::class::Type;
use super::super::object::Object;
use super::BUILTIN_TYPES;

/// Built in boolean type
#[derive(Debug, PartialEq)]
pub struct Bool {
    value: bool,
}

impl fmt::Display for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<bool> for Bool {
    fn from(value: bool) -> Self {
        Bool { value }
    }
}

impl Object for Bool {
    fn class(&self) -> Arc<Type> {
        BUILTIN_TYPES.get("Bool").unwrap().clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
