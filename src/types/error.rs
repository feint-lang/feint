//! # Error Type
//!
//! The error type represents _recoverable_ runtime errors that can be
//! checked in user code using this pattern:
//!
//! result = assert(false)
//! if result.err ->
//!     # Handle `result` as an `Error`
//!     print(result)
//!
//! _All_ objects respond to `err`, which returns either an `Error`
//! object or `nil`. `Error` objects evaluate as `false` in a boolean
//! context:
//!
//! if !assert(false) ->
//!     print("false is not true")
use indexmap::IndexMap;
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::{RuntimeBoolResult, RuntimeErr};

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::error_type::ErrorTypeObj;
use super::map::Map;
use super::ns::Namespace;

static ERROR_TYPES: Lazy<new::obj_ref_t!(Map)> = Lazy::new(|| {
    let names = vec!["assertion", "not_error"];
    let mut types: IndexMap<String, ObjectRef> = IndexMap::new();
    for name in names.into_iter() {
        types.insert(name.to_owned(), new::obj_ref!(ErrorTypeObj::new(name)));
    }
    let types = Map::new(types);
    new::obj_ref!(types)
});

// Error Type ------------------------------------------------------------

gen::type_and_impls!(ErrorType, Error);

pub static ERROR_TYPE: Lazy<new::obj_ref_t!(ErrorType)> = Lazy::new(|| {
    let type_ref = new::obj_ref!(ErrorType::new());
    let mut class = type_ref.write().unwrap();

    class.ns_mut().add_entries(&[
        // Instance Attributes -----------------------------------------
        ("types", ERROR_TYPES.clone()),
        gen::prop!("type", type_ref, |this, _, _| {
            let this = this.read().unwrap();
            let this = this.down_to_error().unwrap();
            let types = ERROR_TYPES.read().unwrap();
            let error_type = match this.kind {
                ErrorKind::Assertion => types.get("assertion"),
                ErrorKind::NotError => types.get("not_error"),
            };
            if let Some(error_type) = error_type {
                Ok(error_type.clone())
            } else {
                Err(RuntimeErr::arg_err(""))
            }
        }),
    ]);

    type_ref.clone()
});

// Error Object --------------------------------------------------------

#[derive(PartialEq)]
pub enum ErrorKind {
    Assertion,
    NotError,
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

    pub fn new_assertion_error(message: String) -> Self {
        Self { ns: Namespace::new(), kind: ErrorKind::Assertion, message }
    }

    pub fn new_not_error() -> Self {
        Self { ns: Namespace::new(), kind: ErrorKind::NotError, message: "".to_owned() }
    }
}

impl ObjectTrait for Error {
    gen::object_trait_header!(ERROR_TYPE);

    fn bool_val(&self) -> RuntimeBoolResult {
        Ok(self.kind != ErrorKind::NotError)
    }

    fn and(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        let lhs = self.bool_val()?;
        let rhs = rhs.bool_val()?;
        Ok(lhs && rhs)
    }

    fn or(&self, rhs: &dyn ObjectTrait) -> RuntimeBoolResult {
        let lhs = self.bool_val()?;
        let rhs = rhs.bool_val()?;
        Ok(lhs || rhs)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Assertion => "Assertion failed",
            Self::NotError => "Not an error",
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
        if self.kind == ErrorKind::NotError {
            write!(f, "{}", self.kind)
        } else if self.message.is_empty() {
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
