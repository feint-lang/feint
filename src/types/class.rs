//! "Class" and "type" are used interchangeably and mean exactly the
//! same thing. Lower case "class" is used instead of "type" because the
//! latter is a Rust keyword.
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

pub type TypeRef = Rc<Type>;

pub struct Func {
    module: String,
    name: String,
    params: Vec<String>,
}

/// Represents a type, whether builtin or user-defined.
pub struct Type {
    module: String,
    name: String,
    methods: HashMap<String, Func>,
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
