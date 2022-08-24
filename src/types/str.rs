use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::{RuntimeBoolResult, RuntimeErr, RuntimeObjResult};

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Str Type ------------------------------------------------------------

gen::type_and_impls!(StrType, Str);

pub static STR_TYPE: Lazy<new::obj_ref_t!(StrType)> = Lazy::new(|| {
    let type_ref = new::obj_ref!(StrType::new());
    let mut class = type_ref.write().unwrap();

    class.ns_mut().add_entries(&[
        // Instance Attributes -----------------------------------------
        gen::prop!("length", type_ref, |this, _, _| {
            let this = this.read().unwrap();
            let this = this.down_to_str().unwrap();
            Ok(new::int(this.value.len()))
        }),
        // Instance Methods --------------------------------------------
        gen::meth!("starts_with", type_ref, &["prefix"], |this, args, _| {
            let this = this.read().unwrap();
            let this = this.down_to_str().unwrap();
            let arg = gen::use_arg!(args, 0);
            let prefix = gen::use_arg_str!(arg);
            Ok(new::bool(this.value.starts_with(prefix)))
        }),
    ]);

    type_ref.clone()
});

// Str Object ----------------------------------------------------------

pub struct Str {
    ns: Namespace,
    value: String,
}

gen::standard_object_impls!(Str);

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
    gen::object_trait_header!(STR_TYPE);

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
