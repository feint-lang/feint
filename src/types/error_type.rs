//! Error Type Type
//!
//! This is a builtin type used to tag builtin `Error` instances.
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Error Type Type -----------------------------------------------------

gen::type_and_impls!(ErrorTypeType, ErrorType);

pub static ERROR_TYPE_TYPE: Lazy<new::obj_ref_t!(ErrorTypeType)> = Lazy::new(|| {
    let type_ref = new::obj_ref!(ErrorTypeType::new());
    let mut class = type_ref.write().unwrap();

    class.ns_mut().add_entries(&[
        // Instance Attributes -----------------------------------------
        gen::prop!("name", type_ref, |this, _, _| {
            let this = this.read().unwrap();
            let this = this.as_any().downcast_ref::<ErrorTypeObj>().unwrap();
            Ok(new::str(&this.name))
        }),
    ]);

    type_ref.clone()
});

// Error Type Object ---------------------------------------------------

pub struct ErrorTypeObj {
    ns: Namespace,
    name: String,
}

gen::standard_object_impls!(ErrorTypeObj);

impl ErrorTypeObj {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { ns: Namespace::new(), name: name.into() }
    }
}

impl ObjectTrait for ErrorTypeObj {
    gen::object_trait_header!(ERROR_TYPE_TYPE);
}

// Display -------------------------------------------------------------

impl fmt::Display for ErrorTypeObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Debug for ErrorTypeObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
