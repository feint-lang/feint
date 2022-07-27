//! "Class" and "type" are used interchangeably and mean exactly the
//! same thing. Lower case "class" is used instead of "type" because the
//! latter is a Rust keyword.
use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::vm::{RuntimeContext, RuntimeErr};

use super::builtin_types::BUILTIN_TYPES;
use super::object::Object;
use super::result::GetAttributeResult;

pub type TypeRef = Arc<Type>;

/// Represents a type, whether builtin or user-defined.
#[derive(Clone)]
pub struct Type {
    module: String,
    name: String,
}

impl Type {
    pub fn new<S: Into<String>>(module: S, name: S) -> Self {
        Self { module: module.into(), name: name.into() }
    }

    pub fn id(&self) -> usize {
        self as *const Self as usize
    }

    pub fn module(&self) -> String {
        self.module.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.module, self.name)
    }

    pub fn is(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Object for Type {
    fn class(&self) -> &TypeRef {
        BUILTIN_TYPES.get("Type").unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_attribute(&self, name: &str, ctx: &RuntimeContext) -> GetAttributeResult {
        match name {
            "module" => Ok(ctx.builtins.new_string(self.module())),
            "name" => Ok(ctx.builtins.new_string(self.type_name())),
            "id" => Ok(ctx.builtins.new_int(self.id())),
            _ => Err(RuntimeErr::new_attribute_does_not_exit(name)),
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.is(other)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Type: {}]", self.qualified_name())
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
