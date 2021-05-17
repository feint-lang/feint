//! The arena is where objects are stored. The VM stack references
//! objects by pointer.

use crate::builtins::Object;

pub struct Arena<'a> {
    storage: Vec<Object<'a>>,
}
