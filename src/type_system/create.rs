//! Type Constructors.
//!
//! These constructors simplify the creation system objects.
use std::sync::Arc;

use crate::type_system::module::Module;
use num_bigint::BigInt;
use once_cell::sync::Lazy;

use super::base::ObjectRef;
use super::bool::Bool;
use super::int::Int;
use super::nil::Nil;
use super::ns::Namespace;
use super::str::Str;

static NIL: Lazy<Arc<Nil>> = Lazy::new(|| Arc::new(Nil::new()));
static TRUE: Lazy<Arc<Bool>> = Lazy::new(|| Arc::new(Bool::new(true)));
static FALSE: Lazy<Arc<Bool>> = Lazy::new(|| Arc::new(Bool::new(false)));

pub fn new_nil() -> ObjectRef {
    NIL.clone()
}

pub fn new_true() -> ObjectRef {
    TRUE.clone()
}

pub fn new_false() -> ObjectRef {
    FALSE.clone()
}

pub fn new_int(value: BigInt) -> ObjectRef {
    Arc::new(Int::new(value))
}

pub fn new_int_from_usize(value: usize) -> ObjectRef {
    Arc::new(Int::from_usize(value))
}

pub fn new_module<S: Into<String>>(name: S, ns: Arc<Namespace>) -> ObjectRef {
    Arc::new(Module::new(name, ns))
}

pub fn new_namespace() -> ObjectRef {
    Arc::new(Namespace::new())
}

pub fn new_str<S: Into<String>>(value: S) -> ObjectRef {
    Arc::new(Str::new(value))
}
