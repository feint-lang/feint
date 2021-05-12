use std::collections::HashMap;
use std::fmt;

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
        Object { name: name.to_owned(), type_: &self, attributes }
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

#[derive(Debug)]
pub struct Object<'a> {
    pub name: String,
    pub type_: &'a Type<'a>,
    pub attributes: HashMap<&'a str, &'a Object<'a>>,
}

impl<'a> Object<'a> {
    pub fn id(&self) -> *const Self {
        self as *const Self
    }
}

impl<'a> fmt::Display for Object<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({:?}) -> {} ({:?})",
            self.type_.name,
            self.type_.id(),
            self.name,
            self.id()
        )
    }
}

impl<'a> PartialEq for Object<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

#[derive(Debug)]
pub struct Method {
    name: String,
    // ???
}

pub struct IntType {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_type() {
        let t = Type::new("Type", HashMap::new());
        println!("{}", t);
    }

    #[test]
    fn make_object() {
        let t = Type::new("Type", HashMap::new());
        let o = t.new_instance("Object", HashMap::new());
        println!("{}", o);
    }
}
