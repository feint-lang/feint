use std::any::Any;
use std::fmt;
use std::sync::Arc;

use num_bigint::BigInt;
use num_traits::FromPrimitive;

use once_cell::sync::Lazy;

use super::base::{ObjectRef, ObjectTrait, TypeTrait};
use super::class::TYPE;
use super::create;
use super::ns::Namespace;

// Int Type ------------------------------------------------------------

static INT_TYPE: Lazy<Arc<IntType>> = Lazy::new(|| Arc::new(IntType::new()));

pub struct IntType {
    namespace: Arc<Namespace>,
}

unsafe impl Send for IntType {}
unsafe impl Sync for IntType {}

impl IntType {
    pub fn new() -> Self {
        let mut ns = Namespace::new();
        ns.add_obj("$module", create::new_str("builtins"));
        ns.add_obj("$name", create::new_str("Int"));
        ns.add_obj("$full_name", create::new_str("builtins.Int"));
        Self { namespace: Arc::new(ns) }
    }
}

impl TypeTrait for IntType {
    fn name(&self) -> &str {
        "Int"
    }

    fn full_name(&self) -> &str {
        "builtins.Int"
    }
}

impl ObjectTrait for IntType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> ObjectRef {
        TYPE.clone()
    }

    fn namespace(&self) -> ObjectRef {
        self.namespace.clone()
    }
}

// Int Object ----------------------------------------------------------

pub struct Int {
    namespace: Arc<Namespace>,
    value: BigInt,
}

unsafe impl Send for Int {}
unsafe impl Sync for Int {}

impl Int {
    pub fn new(value: BigInt) -> Self {
        Self { namespace: Arc::new(Namespace::new()), value }
    }

    pub fn from_usize(value: usize) -> Self {
        Self::new(BigInt::from_usize(value).unwrap())
    }

    pub fn value(&self) -> &BigInt {
        &self.value
    }
}

impl ObjectTrait for Int {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> ObjectRef {
        INT_TYPE.clone()
    }

    fn namespace(&self) -> ObjectRef {
        self.namespace.clone()
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for IntType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}.{}>", self.module(), self.name())
    }
}

impl fmt::Debug for IntType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Debug for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
