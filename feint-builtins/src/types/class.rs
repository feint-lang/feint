//! "Class" and "type" are used interchangeably and mean exactly the
//! same thing. Lower case "class" is used instead of "type" because the
//! latter is a Rust keyword.
use std::any::Any;
use std::fmt;

use feint_code_gen::*;

use crate::BUILTINS;

use super::base::{ObjectRef, ObjectTrait, TypeRef};
use super::ns::Namespace;

pub struct Type {
    module_name: String,
    name: String,
    ns: Namespace,
}

standard_object_impls!(Type);

impl Type {
    pub fn new(module_name: &str, name: &str) -> Self {
        Self {
            module_name: module_name.to_owned(),
            name: name.to_owned(),
            ns: Namespace::default(),
        }
    }

    pub fn module_name(&self) -> &String {
        &self.module_name
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

impl ObjectTrait for Type {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        BUILTINS.type_type()
    }

    fn type_obj(&self) -> ObjectRef {
        BUILTINS.type_type()
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
    }

    fn get_attr(&self, name: &str, this: ObjectRef) -> ObjectRef {
        // TODO: Don't recreate attrs on every access
        match name {
            "$module" => BUILTINS.str(&self.module_name),
            "$name" => BUILTINS.str(&self.name),
            "$full_name" => BUILTINS.str(format!("{}.{}", self.module_name, self.name)),
            _ => self.base_get_attr(name, this),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<type {}>", self.name)
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<type {}.{} @ {}>", self.module_name, self.name, self.id())
    }
}
