use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::vm::{RuntimeBoolResult, RuntimeContext, RuntimeErr};
use once_cell::sync::Lazy;

use super::create;

use super::base::{ObjectRef, ObjectTrait, ObjectTraitExt, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Bool Type -----------------------------------------------------------

pub static BOOL_TYPE: Lazy<Arc<BoolType>> = Lazy::new(|| Arc::new(BoolType::new()));

pub struct BoolType {
    namespace: Arc<Namespace>,
}

unsafe impl Send for BoolType {}
unsafe impl Sync for BoolType {}

impl BoolType {
    pub fn new() -> Self {
        let mut ns = Namespace::new();
        ns.add_obj("$name", create::new_str("Bool"));
        ns.add_obj("$full_name", create::new_str("builtins.Bool"));
        Self { namespace: Arc::new(ns) }
    }
}

impl TypeTrait for BoolType {
    fn name(&self) -> &str {
        "Bool"
    }

    fn full_name(&self) -> &str {
        "builtins.Bool"
    }
}

impl ObjectTrait for BoolType {
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

// Bool Object ---------------------------------------------------------

pub struct Bool {
    namespace: Arc<Namespace>,
    value: bool,
}

unsafe impl Send for Bool {}
unsafe impl Sync for Bool {}

impl Bool {
    pub fn new(value: bool) -> Self {
        Self { namespace: Arc::new(Namespace::new()), value }
    }

    pub fn value(&self) -> &bool {
        &self.value
    }
}

impl ObjectTrait for Bool {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        BOOL_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        BOOL_TYPE.clone()
    }

    fn namespace(&self) -> ObjectRef {
        self.namespace.clone()
    }

    // Unary operations -----------------------------------------------

    fn bool_val(&self, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        Ok(*self.value())
    }

    // Binary operations -----------------------------------------------

    fn is_equal(&self, rhs: &dyn ObjectTrait, _ctx: &RuntimeContext) -> bool {
        if let Some(rhs) = rhs.down_to_bool() {
            self.is(rhs) || self.value() == rhs.value()
        } else {
            false
        }
    }

    fn and(&self, rhs: &dyn ObjectTrait, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_bool() {
            Ok(*self.value() && *rhs.value())
        } else {
            Err(RuntimeErr::new_type_err(format!(
                "{} && {} not implemented",
                self.class(),
                rhs.class()
            )))
        }
    }

    fn or(&self, rhs: &dyn ObjectTrait, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_bool() {
            Ok(*self.value() || *rhs.value())
        } else {
            Err(RuntimeErr::new_type_err(format!(
                "{} || {} not implemented",
                self.class(),
                rhs.class()
            )))
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Debug for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
