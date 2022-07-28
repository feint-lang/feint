//! "Class" and "type" are used interchangeably and mean exactly the
//! same thing. Lower case "class" is used instead of "type" because the
//! latter is a Rust keyword.
use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::vm::RuntimeContext;

use super::builtin_types::BUILTIN_TYPES;
use super::object::{Object, ObjectRef};
use super::result::GetAttrResult;

pub type TypeRef = Arc<Type>;

/// Represents a type, whether builtin or user-defined.
#[derive(Clone)]
pub struct Type {
    module: String,
    name: String,
    qualified_name: String,
}

impl Type {
    pub fn new<S: Into<String>>(module: S, name: S) -> Self {
        let module = module.into();
        let name = name.into();
        let qualified_name = format!("{}.{}", module, name);
        Self { module, name, qualified_name }
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
        self.qualified_name.clone()
    }

    pub fn is(&self, other: &Self) -> bool {
        self.id() == other.id()
    }

    // Attributes ------------------------------------------------------

    fn get_tuple_attr(&self, name: &str, ctx: &RuntimeContext) -> Option<ObjectRef> {
        let attr = match name {
            "new" => ctx.builtins.new_tuple(vec![]),
            _ => return None,
        };
        Some(attr)
    }
}

impl Object for Type {
    fn class(&self) -> &TypeRef {
        BUILTIN_TYPES.get("Type").unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_attr(&self, name: &str, ctx: &RuntimeContext) -> GetAttrResult {
        if let Some(attr) = self.get_base_attr(name, ctx) {
            return Ok(attr);
        }
        let attr = match name {
            "module" => ctx.builtins.new_str(self.module()),
            "name" => ctx.builtins.new_str(self.name()),
            "qualified_name" => ctx.builtins.new_str(self.qualified_name()),
            _ => {
                let attr = match self.qualified_name.as_str() {
                    "builtins.Tuple" => self.get_tuple_attr(name, ctx),
                    _ => return Err(self.attr_does_not_exist(name)),
                };
                if let Some(attr) = attr {
                    attr
                } else {
                    return Err(self.attr_does_not_exist(name));
                }
            }
        };
        Ok(attr)
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
