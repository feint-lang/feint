//! # Error Type
//!
//! The error type represents _recoverable_ runtime errors that can be
//! checked in user code using this pattern:
//!
//!     result = do_something()
//!     if result.err ->
//!         # Handle `result` as an `Error`
//!         print(result)
//!
//! _All_ objects respond to `err`, which returns either an `Error`
//! object or `nil`. `Error` objects evaluate as `false` in a boolean
//! context:
//!
//!     if !assert(false) ->
//!         print("false is not true")
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::RuntimeBoolResult;

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Error Type ------------------------------------------------------------

gen::type_and_impls!(ErrorType, Error);

pub static ERROR_TYPE: Lazy<new::obj_ref_t!(ErrorType)> = Lazy::new(|| {
    let type_ref = new::obj_ref!(ErrorType::new());
    let mut class = type_ref.write().unwrap();

    class.ns_mut().add_entries(&[
        // Class Methods -----------------------------------------------
        gen::meth!("new", type_ref, &[], |_, _, _| Ok(new::nil())),
    ]);

    type_ref.clone()
});

// Error Object --------------------------------------------------------

pub enum ErrorKind {
    Assertion,
}

pub struct Error {
    ns: Namespace,
    kind: ErrorKind,
    message: String,
}

gen::standard_object_impls!(Error);

impl Error {
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self { ns: Namespace::new(), kind, message }
    }
}

impl ObjectTrait for Error {
    gen::object_trait_header!(ERROR_TYPE);

    fn bool_val(&self) -> RuntimeBoolResult {
        Ok(false)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Assertion => "Assertion failed",
        };
        write!(f, "{message}")
    }
}

impl fmt::Debug for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.message.is_empty() {
            write!(f, "ERROR: {}", self.kind)
        } else {
            write!(f, "ERROR: {}: {}", self.kind, self.message)
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
