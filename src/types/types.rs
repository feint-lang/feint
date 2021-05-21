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
    slots: Option<Vec<String>>,
    methods: HashMap<String, Method>,
}

impl Type {
    pub fn new<S: Into<String>>(module: S, name: S, slots: Option<Vec<&str>>) -> Self {
        Self {
            module: module.into(),
            name: name.into(),
            slots: slots.map_or(None, |slots| {
                Some(slots.iter().map(|s| String::from(*s)).collect())
            }),
            methods: HashMap::new(),
        }
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

    pub fn has_slot(&self, name: &str) -> bool {
        self.slots
            .as_ref()
            .map_or(true, |slots| slots.iter().position(|n| n == name).is_some())
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
