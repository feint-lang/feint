use std::fmt;
use std::rc::Rc;

use builtin_object_derive::BuiltinObject;

use super::super::class::Type;
use super::super::object::Object;

/// Built in boolean type
#[derive(Debug, PartialEq, BuiltinObject)]
pub struct Bool {
    class: Rc<Type>,
    value: bool,
}

impl Bool {
    pub fn new(class: Rc<Type>, value: bool) -> Self {
        Self { class: class.clone(), value }
    }

    pub fn value(&self) -> &bool {
        &self.value
    }
}

// Binary operations ---------------------------------------------------

impl PartialEq<dyn Object> for Bool {
    fn eq(&self, rhs: &dyn Object) -> bool {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Bool>() {
            self == rhs
        } else {
            panic!("Could not compare Bool to {}", rhs.class());
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
