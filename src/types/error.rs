use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::gen;
use super::meth::make_meth;
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
        // Class Methods
        make_meth!("new", type_ref, &[], |_, _, _| Ok(new::nil())),
    ]);

    type_ref.clone()
});

// Error Object --------------------------------------------------------

#[derive(Clone, Debug)]
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

impl Clone for Error {
    fn clone(&self) -> Self {
        Self::new(self.kind.clone(), self.message.clone())
    }
}

impl ObjectTrait for Error {
    gen::object_trait_header!(ERROR_TYPE);
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
