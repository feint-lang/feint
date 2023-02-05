//! Builtin `Type` type--the base of the type hierarchy.
//!
//! "Class" and "type" are synonymous and used interchangeably. Lower
//! case "class" is used instead of "type" because the latter is a Rust
//! keyword.
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use feint_code_gen::*;

use crate::modules::get_module;
use crate::new;

use super::base::{ObjectTrait, TypeRef};
use super::ns::Namespace;

std_type!(TYPE_TYPE, Type);

pub struct Type {
    module_name: String,
    name: String,
    full_name: String,
    ns: Namespace,
}

standard_object_impls!(Type);

impl Type {
    pub fn new(module_name: &str, name: &str) -> Self {
        let full_name = format!("{module_name}.{name}");
        let full_name_str = new::str(&full_name);
        Self {
            module_name: module_name.to_owned(),
            name: name.to_owned(),
            full_name,
            ns: Namespace::with_entries(&[
                ("$module", get_module(module_name)),
                ("$name", new::str(name)),
                ("$full_name", full_name_str),
            ]),
        }
    }

    pub fn module_name(&self) -> &String {
        &self.module_name
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn full_name(&self) -> &String {
        &self.full_name
    }
}

impl ObjectTrait for Type {
    object_trait_header!(TYPE_TYPE);
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<type {}>", self.name)
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<type {} @ {}>", self.full_name(), self.id())
    }
}
