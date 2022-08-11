use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::{RuntimeBoolResult, RuntimeErr};

use super::create;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Bool Type -----------------------------------------------------------

pub static BOOL_TYPE: Lazy<Arc<RwLock<BoolType>>> =
    Lazy::new(|| Arc::new(RwLock::new(BoolType::new())));

pub struct BoolType {
    namespace: Namespace,
}

unsafe impl Send for BoolType {}
unsafe impl Sync for BoolType {}

impl BoolType {
    pub fn new() -> Self {
        Self {
            namespace: Namespace::with_entries(&[
                // Class Attributes
                ("$name", create::new_str("Bool")),
                ("$full_name", create::new_str("builtins.Bool")),
            ]),
        }
    }
}

impl TypeTrait for BoolType {
    fn name(&self) -> &str {
        "Bool"
    }

    fn full_name(&self) -> &str {
        "builtins.Bool"
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
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

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

// Bool Object ---------------------------------------------------------

pub struct Bool {
    namespace: Namespace,
    value: bool,
}

unsafe impl Send for Bool {}
unsafe impl Sync for Bool {}

impl Bool {
    pub fn new(value: bool) -> Self {
        Self { namespace: Namespace::new(), value }
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

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    // Unary operations -----------------------------------------------

    fn bool_val(&self) -> RuntimeBoolResult {
        Ok(*self.value())
    }

    // Binary operations -----------------------------------------------

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if let Some(rhs) = rhs.down_to_bool() {
            self.is(rhs) || self.value() == rhs.value()
        } else {
            false
        }
    }

    fn and(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_bool() {
            Ok(*self.value() && *rhs.value())
        } else {
            Err(RuntimeErr::type_err(format!(
                "{} && {} not implemented",
                self.class().read().unwrap(),
                rhs.class().read().unwrap(),
            )))
        }
    }

    fn or(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_bool() {
            Ok(*self.value() || *rhs.value())
        } else {
            Err(RuntimeErr::type_err(format!(
                "{} || {} not implemented",
                self.class().read().unwrap(),
                rhs.class().read().unwrap(),
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
