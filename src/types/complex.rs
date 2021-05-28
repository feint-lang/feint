//! A complex object may have builtin objects and other custom/complex
//! objects as attributes. This is opposed to fundamental/builtin types,
//! like `Bool` and `Float` that wrap Rust primitives.
use std::any::Any;
use std::collections::HashMap;
use std::fmt;

use crate::vm::{RuntimeBoolResult, RuntimeContext, RuntimeError, RuntimeErrorKind};

use super::class::TypeRef;
use super::object::{Object, ObjectExt, ObjectRef};

pub type Attributes = HashMap<String, ObjectRef>;

pub struct ComplexObject {
    class: TypeRef,
    attributes: Attributes,
}

impl ComplexObject {
    pub fn new(class: TypeRef) -> Self {
        Self { class: class.clone(), attributes: HashMap::new() }
    }
}

impl Object for ComplexObject {
    fn class(&self) -> &TypeRef {
        &self.class
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_equal(&self, rhs: ObjectRef, ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            Ok(self.is(rhs) || attributes_equal(&self.attributes, &rhs.attributes, ctx)?)
        } else {
            Err(RuntimeError::new_type_error(format!(
                "Could not compare {} to {}",
                self.class().name(),
                rhs.class().name()
            )))
        }
    }

    fn get_attribute(&self, name: &str) -> Result<&ObjectRef, RuntimeError> {
        if let Some(value) = self.attributes.get(name) {
            return Ok(value);
        }
        Err(RuntimeError::new(RuntimeErrorKind::AttributeDoesNotExist(name.to_owned())))
    }

    fn set_attribute(
        &mut self,
        name: &str,
        value: ObjectRef,
    ) -> Result<(), RuntimeError> {
        self.attributes.insert(name.to_owned(), value.clone());
        Ok(())
    }
}

// Display -------------------------------------------------------------

impl fmt::Debug for ComplexObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object {} @ {}", self, self.id())
    }
}

impl fmt::Display for ComplexObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<String> =
            self.attributes.iter().map(|(n, v)| format!("{}={}", n, v)).collect();
        write!(f, "{}({})", self.class.name(), names.join(", "))
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
    if !(lhs.len() == rhs.len() && lhs.keys().all(|k| rhs.contains_key(k))) {
        return Ok(false);
    }
    for (k, v) in lhs.iter() {
        if !v.is_equal(rhs[k].clone(), ctx)? {
            return Ok(false);
        }
    }
    Ok(true)
}
