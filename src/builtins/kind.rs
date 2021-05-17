use std::collections::HashMap;
use std::fmt;

use super::{Method, Object};

#[derive(Debug)]
pub struct Type<'a> {
    pub module: &'a str,
    pub name: &'a str,
    // Attribute slots. These are the names of the  attributes that can
    // be set when creating an instance of a type.
    pub slots: Vec<&'a str>,
    pub methods: HashMap<&'a str, &'a Method>,
}

impl<'a> Type<'a> {
    pub fn new(
        module: &'a str,
        name: &'a str,
        slots: Vec<&'a str>,
        methods: HashMap<&'a str, &'a Method>,
    ) -> Self {
        Type { module, name, slots, methods }
    }

    pub fn new_instance(&self, attributes: HashMap<&'a str, &'a Object>) -> Object {
        Object { kind: self, attributes }
    }

    pub fn id(&self) -> *const Self {
        self as *const Self
    }
}

impl<'a> PartialEq for Type<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<'a> fmt::Display for Type<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({:?})", self.name, self.id())
    }
}
