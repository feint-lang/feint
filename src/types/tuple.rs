//! Built in tuple type
use std::any::Any;
use std::fmt;

use crate::vm::{
    RuntimeBoolResult, RuntimeContext, RuntimeErr, RuntimeErrKind, RuntimeResult,
};

use crate::types::class::TypeRef;
use crate::types::object::{Object, ObjectExt, ObjectRef};

pub struct Tuple {
    class: TypeRef,
    items: Vec<ObjectRef>,
}

impl Tuple {
    pub fn new(class: TypeRef, items: Vec<ObjectRef>) -> Self {
        Self { class, items }
    }

    pub fn items(&self) -> &Vec<ObjectRef> {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl Object for Tuple {
    fn class(&self) -> &TypeRef {
        &self.class
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_equal(&self, rhs: ObjectRef, ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            if self.is(rhs) {
                return Ok(true);
            }
            if self.len() != rhs.len() {
                return Ok(false);
            }
            for (i, j) in self.items().iter().zip(rhs.items()) {
                if !i.is_equal(j.clone(), ctx)? {
                    return Ok(false);
                }
            }
            return Ok(true);
        } else {
            Err(RuntimeErr::new_type_error(format!(
                "Could not compare Tuple to {} for equality",
                rhs.class().name()
            )))
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items: Vec<String> =
            self.items().iter().map(|i| format!("{:?}", i)).collect();
        write!(f, "({})", items.join(", "))
    }
}

impl fmt::Debug for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
