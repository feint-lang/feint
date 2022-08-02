use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::vm::{RuntimeBoolResult, RuntimeContext, RuntimeErr, RuntimeObjResult};
use once_cell::sync::Lazy;

use super::create;

use super::base::{ObjectRef, ObjectTrait, ObjectTraitExt, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Str Type ------------------------------------------------------------

pub static STR_TYPE: Lazy<Arc<StrType>> = Lazy::new(|| Arc::new(StrType::new()));

pub struct StrType {
    namespace: Arc<Namespace>,
}

impl StrType {
    pub fn new() -> Self {
        let mut ns = Namespace::new();
        ns.add_obj("$name", create::new_str("Str"));
        ns.add_obj("$full_name", create::new_str("builtins.Str"));
        Self { namespace: Arc::new(ns) }
    }
}

unsafe impl Send for StrType {}
unsafe impl Sync for StrType {}

impl TypeTrait for StrType {
    fn name(&self) -> &str {
        "Str"
    }

    fn full_name(&self) -> &str {
        "builtins.Str"
    }
}

impl ObjectTrait for StrType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        TYPE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        TYPE_TYPE.clone()
    }

    fn namespace(&self) -> ObjectRef {
        self.namespace.clone()
    }
}

// Str Object ----------------------------------------------------------

pub struct Str {
    namespace: Arc<Namespace>,
    value: String,
}

impl Str {
    pub fn new<S: Into<String>>(value: S) -> Self {
        Self { namespace: Arc::new(Namespace::new()), value: value.into() }
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

impl ObjectTrait for Str {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        STR_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        STR_TYPE.clone()
    }

    fn namespace(&self) -> ObjectRef {
        self.namespace.clone()
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait, _ctx: &RuntimeContext) -> bool {
        if let Some(rhs) = rhs.down_to_str() {
            self.is(rhs) || self.value() == rhs.value()
        } else {
            false
        }
    }

    fn add(&self, rhs: &dyn ObjectTrait, ctx: &RuntimeContext) -> RuntimeObjResult {
        if let Some(rhs) = rhs.down_to_str() {
            let a = self.value();
            let b = rhs.value();
            let mut value = String::with_capacity(a.len() + b.len());
            value.push_str(a);
            value.push_str(b);
            let value = ctx.builtins.new_str(value);
            Ok(value)
        } else {
            Err(RuntimeErr::new_type_err(format!(
                "Cannot concatenate {} to {}",
                rhs.class(),
                self.class(),
            )))
        }
    }

    fn less_than(
        &self,
        rhs: &dyn ObjectTrait,
        _ctx: &RuntimeContext,
    ) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_str() {
            Ok(self.value() < rhs.value())
        } else {
            Err(RuntimeErr::new_type_err(format!(
                "Cannot compare {} to {}: <",
                rhs.class(),
                self.class(),
            )))
        }
    }

    fn greater_than(
        &self,
        rhs: &dyn ObjectTrait,
        _ctx: &RuntimeContext,
    ) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_str() {
            Ok(self.value() > rhs.value())
        } else {
            Err(RuntimeErr::new_type_err(format!(
                "Cannot compare {} to {}: >",
                rhs.class(),
                self.class(),
            )))
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Debug for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}
