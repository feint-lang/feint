//! A custom object may have builtin objects and other custom objects as
//! attributes. This is opposed to fundamental/builtin types, like
//! `Bool` and `Float` that wrap Rust primitives.
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use crate::vm::{RuntimeContext, RuntimeErr};

use super::class::TypeRef;
use super::object::{Object, ObjectExt, ObjectRef};
use super::result::{GetAttrResult, SetAttrResult};

pub type Attrs = RefCell<HashMap<String, ObjectRef>>;

pub struct Custom {
    class: TypeRef,
    attrs: Attrs,
}

impl Custom {
    pub fn new(class: TypeRef) -> Self {
        Self { class, attrs: RefCell::new(HashMap::new()) }
    }
}

impl Object for Custom {
    fn class(&self) -> &TypeRef {
        &self.class
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_equal(&self, rhs: &dyn Object, _ctx: &RuntimeContext) -> bool {
        // let rhs = rhs.lock().unwrap();
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            self.is(&rhs)
                || (self.class().lock().unwrap().is(&rhs.class().lock().unwrap())
                    && attrs_equal(&self.attrs, &rhs.attrs, _ctx))
        } else {
            false
        }
    }

    fn get_attr(&self, name: &str, _ctx: &RuntimeContext) -> GetAttrResult {
        if let Some(value) = self.attrs.borrow().get(name) {
            return Ok(value.clone());
        }
        Err(RuntimeErr::new_attr_does_not_exist(
            self.qualified_type_name().as_str(),
            name,
        ))
    }

    fn set_attr(
        &self,
        name: &str,
        value: ObjectRef,
        _ctx: &RuntimeContext,
    ) -> SetAttrResult {
        self.attrs.borrow_mut().insert(name.to_owned(), value.clone());
        Ok(())
    }
}

// Util ----------------------------------------------------------------

/// Compare attributes for equality. The attribute maps are first
/// checked to see if they have the same number of entries. Then, the
/// keys are checked to see if they're all the same. If they are, only
/// then are the values checked for equality.
fn attrs_equal(lhs: &Attrs, rhs: &Attrs, ctx: &RuntimeContext) -> bool {
    let lhs = lhs.borrow();
    let rhs = rhs.borrow();
    if !(lhs.len() == rhs.len() && lhs.keys().all(|k| rhs.contains_key(k))) {
        return false;
    }
    for (k, v) in lhs.iter() {
        let v = v.lock().unwrap();
        if !v.is_equal(&(*rhs[k].lock().unwrap()), ctx) {
            return false;
        }
    }
    true
}

// Display -------------------------------------------------------------

impl fmt::Display for Custom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let attrs: Vec<String> = self
            .attrs
            .borrow()
            .iter()
            .map(|(n, v)| format!("{}={}", n, v.lock().unwrap()))
            .collect();
        write!(f, "{}({})", self.type_name(), attrs.join(", "))
    }
}

impl fmt::Debug for Custom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object {} @ {}", self, self.id())
    }
}
