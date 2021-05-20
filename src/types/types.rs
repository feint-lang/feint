use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use num_bigint::BigInt;

use super::builtins;
use super::Object;

pub struct Method {
    name: String,
    parameters: Vec<String>,
}

pub struct Type {
    module: String,
    name: String,
    slots: Vec<String>,
    methods: HashMap<String, Method>,
}

impl Type {
    pub fn new(module: &str, name: &str, slots: Vec<&str>) -> Self {
        let module = module.to_owned();
        let name = name.to_owned();
        let slots = slots.iter().map(|s| (*s).to_owned()).collect();
        Self { module, name, slots, methods: HashMap::new() }
    }

    pub fn id(&self) -> *const Self {
        self as *const Self
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

    pub fn is_equal(&self, other: &Self) -> bool {
        other.is(self)
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.is(other)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Type {} @ {:?}", self.name(), self.id())
    }
}
