use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::RuntimeBoolResult;

use super::meth::make_meth;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Error Type ------------------------------------------------------------

pub static ERROR_TYPE: Lazy<Arc<RwLock<ErrorType>>> = Lazy::new(|| {
    let type_ref = Arc::new(RwLock::new(ErrorType::new()));
    let mut class = type_ref.write().unwrap();

    class.ns_mut().add_entries(&[
        // Class Attributes
        ("$name", new::str("Error")),
        ("$full_name", new::str("builtins.Error")),
        // Class Methods
        make_meth!("new", type_ref, &[], |_, _, _| Ok(new::nil())),
    ]);

    type_ref.clone()
});

pub struct ErrorType {
    namespace: Namespace,
}

unsafe impl Send for ErrorType {}
unsafe impl Sync for ErrorType {}

impl ErrorType {
    pub fn new() -> Self {
        Self { namespace: Namespace::new() }
    }
}

impl TypeTrait for ErrorType {
    fn name(&self) -> &str {
        "Error"
    }

    fn full_name(&self) -> &str {
        "builtins.Error"
    }

    fn ns(&self) -> &Namespace {
        &self.namespace
    }
}

impl ObjectTrait for ErrorType {
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
        &self.namespace
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.namespace
    }
}

// Error Object --------------------------------------------------------

#[derive(Clone, Debug)]
pub enum ErrorKind {
    Assertion,
}

pub struct Error {
    namespace: Namespace,
    kind: ErrorKind,
    message: String,
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl Error {
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self { namespace: Namespace::new(), kind, message }
    }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        Self::new(self.kind.clone(), self.message.clone())
    }
}

impl ObjectTrait for Error {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        ERROR_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        ERROR_TYPE.clone()
    }

    fn ns(&self) -> &Namespace {
        &self.namespace
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.namespace
    }

    fn bool_val(&self) -> RuntimeBoolResult {
        Ok(false)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ERROR: {:?}: {}", self.kind, self.message)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
