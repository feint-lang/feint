//! A complex object may have builtin objects and other custom/complex
//! objects as attributes. This is opposed to fundamental/builtin types,
//! like `Bool` and `Float` that wrap Rust primitives.
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use crate::vm::{RuntimeBoolResult, RuntimeContext, RuntimeErr, RuntimeErrKind};

use super::class::TypeRef;
use super::object::{Object, ObjectExt, ObjectRef};
use super::result::{GetAttributeResult, SetAttributeResult};

pub type Attributes = RefCell<HashMap<String, ObjectRef>>;

pub struct ComplexObject {
    class: TypeRef,
    attributes: Attributes,
}

impl ComplexObject {
    pub fn new(class: TypeRef) -> Self {
        Self { class, attributes: RefCell::new(HashMap::new()) }
    }
}

impl Object for ComplexObject {
    fn class(&self) -> &TypeRef {
        &self.class
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_equal(&self, rhs: &ObjectRef, ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            Ok(self.is(&rhs)
                || (self.class() == rhs.class()
                    && attributes_equal(&self.attributes, &rhs.attributes, ctx)?))
        } else {
            Err(RuntimeErr::new_type_err(format!(
                "Could not compare {} to {}",
                self.class().name(),
                rhs.class().name()
            )))
        }
    }

    fn get_attribute(&self, name: &str, _ctx: &RuntimeContext) -> GetAttributeResult {
        if let Some(value) = self.attributes.borrow().get(name) {
            return Ok(value.clone());
        }
        Err(RuntimeErr::new(RuntimeErrKind::AttributeDoesNotExist(name.to_owned())))
    }

    fn set_attribute(
        &self,
        name: &str,
        value: ObjectRef,
        _ctx: &RuntimeContext,
    ) -> SetAttributeResult {
        self.attributes.borrow_mut().insert(name.to_owned(), value.clone());
        Ok(())
    }
}

// Util ----------------------------------------------------------------

/// Compare attributes for equality. The attribute maps are first
/// checked to see if they have the same number of entries. Then, the
/// keys are checked to see if they're all the same. If they are, only
/// then are the values checked for equality.
fn attributes_equal(
    lhs: &Attributes,
    rhs: &Attributes,
    ctx: &RuntimeContext,
) -> RuntimeBoolResult {
    let lhs = lhs.borrow();
    let rhs = rhs.borrow();
    if !(lhs.len() == rhs.len() && lhs.keys().all(|k| rhs.contains_key(k))) {
        return Ok(false);
    }
    for (k, v) in lhs.iter() {
        if !v.is_equal(&rhs[k], ctx)? {
            return Ok(false);
        }
    }
    Ok(true)
}

// Display -------------------------------------------------------------

impl fmt::Display for ComplexObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let attrs: Vec<String> = self
            .attributes
            .borrow()
            .iter()
            .map(|(n, v)| format!("{}={}", n, v))
            .collect();
        write!(f, "{}({})", self.class.name(), attrs.join(", "))
    }
}

impl fmt::Debug for ComplexObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object {} @ {}", self, self.id())
    }
}
