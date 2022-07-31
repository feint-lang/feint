//! A custom object may have builtin objects and other custom objects as
//! attributes. This is opposed to fundamental/builtin types, like
//! `Bool` and `Float` that wrap Rust primitives.
use std::any::Any;
use std::cell::RefCell;
use std::fmt;

use crate::vm::RuntimeContext;

use super::class::TypeRef;
use super::namespace::Namespace;
use super::object::{Object, ObjectExt, ObjectRef};
use super::result::{GetAttrResult, SetAttrResult};

pub struct Custom {
    class: TypeRef,
    attrs: RefCell<Namespace>,
}

impl Custom {
    pub fn new(class: TypeRef) -> Self {
        Self { class, attrs: RefCell::new(Namespace::new()) }
    }
}

impl Object for Custom {
    fn class(&self) -> &TypeRef {
        &self.class
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_attr(&self, name: &str, _ctx: &RuntimeContext) -> GetAttrResult {
        if let Some(value) = self.attrs.borrow().get_var(name) {
            return Ok(value.clone());
        }
        Err(self.attr_does_not_exist(name))
    }

    fn set_attr(
        &self,
        name: &str,
        value: ObjectRef,
        _ctx: &RuntimeContext,
    ) -> SetAttrResult {
        self.attrs.borrow_mut().add_var(name, value);
        Ok(())
    }

    fn is_equal(&self, rhs: &dyn Object, ctx: &RuntimeContext) -> bool {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            if self.is(&rhs) {
                // Object is equal to itself.
                true
            } else if !self.class().is(&rhs.class()) {
                // Objects are not the same type so they can't be equal.
                false
            } else {
                // Otherwise, objects are the same type, so check their
                // attribute namespaces for equality.
                let lhs_attrs = self.attrs.borrow();
                let rhs_attrs = &*rhs.attrs.borrow();
                lhs_attrs.is_equal(rhs_attrs, ctx)
            }
        } else {
            false
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Custom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Check for $string attr and use that value if present
        let type_name = self.class();
        let id = self.id();
        write!(f, "<{type_name}> object @ {id}")
    }
}

impl fmt::Debug for Custom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
