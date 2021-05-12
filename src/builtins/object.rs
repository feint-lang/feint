use std::collections::HashMap;
use std::fmt;

use super::Type;

#[derive(Debug)]
pub struct Object<'a> {
    pub name: String,
    pub kind: &'a Type<'a>,
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
            self.kind.name,
            self.kind.id(),
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
