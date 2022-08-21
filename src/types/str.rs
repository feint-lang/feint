use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::{RuntimeBoolResult, RuntimeErr, RuntimeObjResult, VM};

use super::meth::{make_meth, use_arg, use_arg_str, use_this};
use super::new;
use super::result::{Args, This};

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Str Type ------------------------------------------------------------

pub static STR_TYPE: Lazy<Arc<RwLock<StrType>>> =
    Lazy::new(|| Arc::new(RwLock::new(StrType::new())));

pub struct StrType {
    ns: Namespace,
}

impl StrType {
    pub fn new() -> Self {
        Self {
            ns: Namespace::with_entries(&[
                // Class Attributes
                ("$name", new::str("Str")),
                ("$full_name", new::str("builtins.Str")),
                make_meth!(
                    Str,
                    "starts_with",
                    &["prefix"],
                    |this: ObjectRef, args: Args, _| {
                        let this = use_this!(this);
                        let this = this.down_to_str().unwrap();
                        let arg = use_arg!(args, 0);
                        let prefix = use_arg_str!(arg);
                        Ok(new::bool(this.value.starts_with(prefix)))
                    }
                ),
            ]),
        }
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

    fn ns(&self) -> &Namespace {
        &self.ns
    }
}

impl ObjectTrait for StrType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn class(&self) -> TypeRef {
        TYPE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        TYPE_TYPE.clone()
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
    }
}

// Str Object ----------------------------------------------------------

pub struct Str {
    ns: Namespace,
    value: String,
}

impl Str {
    pub fn new(value: String) -> Self {
        Self {
            ns: Namespace::with_entries(&[
                // Instance Attributes
                ("len", new::int(value.len())),
            ]),
            value,
        }
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

impl ObjectTrait for Str {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn class(&self) -> TypeRef {
        STR_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        STR_TYPE.clone()
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if let Some(rhs) = rhs.down_to_str() {
            self.is(rhs) || self.value() == rhs.value()
        } else {
            false
        }
    }

    fn add(&self, rhs: &dyn ObjectTrait) -> RuntimeObjResult {
        if let Some(rhs) = rhs.down_to_str() {
            let a = self.value();
            let b = rhs.value();
            let mut value = String::with_capacity(a.len() + b.len());
            value.push_str(a);
            value.push_str(b);
            let value = new::str(value);
            Ok(value)
        } else {
            Err(RuntimeErr::type_err(format!(
                "Cannot concatenate {} to {}",
                self.class().read().unwrap(),
                rhs.class().read().unwrap(),
            )))
        }
    }

    fn less_than(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_str() {
            Ok(self.value() < rhs.value())
        } else {
            Err(RuntimeErr::type_err(format!(
                "Cannot compare {} to {}: <",
                self.class().read().unwrap(),
                rhs.class().read().unwrap(),
            )))
        }
    }

    fn greater_than(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.down_to_str() {
            Ok(self.value() > rhs.value())
        } else {
            Err(RuntimeErr::type_err(format!(
                "Cannot compare {} to {}: >",
                self.class().read().unwrap(),
                rhs.class().read().unwrap(),
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
