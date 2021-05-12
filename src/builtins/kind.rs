use std::collections::HashMap;
use std::fmt;

use super::{Method, Object};

#[derive(Debug)]
pub struct Type<'a> {
    pub name: String,
    pub methods: HashMap<&'a str, &'a Method>,
}

impl<'a> Type<'a> {
    pub fn new(name: &str, methods: HashMap<&'a str, &'a Method>) -> Self {
        Type { name: name.to_owned(), methods }
    }

    pub fn new_instance(
        &self,
        name: &str,
        attributes: HashMap<&'a str, &'a Object>,
    ) -> Object {
        Object { name: name.to_owned(), kind: self, attributes }
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
