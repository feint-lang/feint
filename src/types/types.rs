use std::collections::HashMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::rc::Rc;

use num_bigint::BigInt;

use super::builtins;
use super::ComplexObject;

pub struct Method {
    name: String,
    parameters: Vec<String>,
}

pub struct Type {
    module: String,
    name: String,
    methods: HashMap<String, Method>,
}

impl Type {
    pub fn new<S: Into<String>>(module: S, name: S) -> Self {
        Self { module: module.into(), name: name.into(), methods: HashMap::new() }
    }

    pub fn id(&self) -> usize {
        self as *const Self as usize
    }

    pub fn module(&self) -> &str {
        self.module.as_str()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn is(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.is(other)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Debug for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Type {} @ {:?}", self.name(), self.id())
    }
}
