//! Type Constructors.
//!
//! These constructors simplify the creation system objects.
use std::sync::{Arc, RwLock};

use crate::types::Namespace;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, Num};
use once_cell::sync::Lazy;

use crate::vm::Chunk;

use super::base::ObjectRef;
use super::bool::Bool;
use super::builtin_func::{BuiltinFn, BuiltinFunc};
use super::custom::{CustomObj, CustomType};
use super::float::Float;
use super::func::Func;
use super::int::Int;
use super::module::Module;
use super::nil::Nil;
use super::str::Str;
use super::tuple::Tuple;

use super::result::Params;

static NIL: Lazy<Arc<RwLock<Nil>>> = Lazy::new(|| Arc::new(RwLock::new(Nil::new())));
static TRUE: Lazy<Arc<RwLock<Bool>>> =
    Lazy::new(|| Arc::new(RwLock::new(Bool::new(true))));
static FALSE: Lazy<Arc<RwLock<Bool>>> =
    Lazy::new(|| Arc::new(RwLock::new(Bool::new(false))));

// Builtin type constructors ---------------------------------------

#[inline]
pub fn new_nil() -> ObjectRef {
    NIL.clone()
}

#[inline]
pub fn new_true() -> ObjectRef {
    TRUE.clone()
}

#[inline]
pub fn new_false() -> ObjectRef {
    FALSE.clone()
}

pub fn new_builtin_func<S: Into<String>>(
    name: S,
    params: Option<Vec<S>>,
    func: BuiltinFn,
) -> ObjectRef {
    let params = collect_params(params);
    Arc::new(RwLock::new(BuiltinFunc::new(name, params, func)))
}

pub fn new_float(value: f64) -> ObjectRef {
    Arc::new(RwLock::new(Float::new(value)))
}

pub fn new_float_from_string<S: Into<String>>(value: S) -> ObjectRef {
    let value = value.into();
    let value = value.parse::<f64>().unwrap();
    new_float(value)
}

pub fn new_func<S: Into<String>>(
    name: S,
    params: Option<Vec<S>>,
    chunk: Chunk,
) -> ObjectRef {
    let params = collect_params(params);
    Arc::new(RwLock::new(Func::new(name, params, chunk)))
}

pub fn new_int<I: Into<BigInt>>(value: I) -> ObjectRef {
    let value = value.into();
    Arc::new(RwLock::new(Int::new(value)))
}

pub fn new_int_from_string<S: Into<String>>(value: S) -> ObjectRef {
    let value = value.into();
    if let Ok(value) = BigInt::from_str_radix(value.as_ref(), 10) {
        new_int(value)
    } else {
        let value = value.parse::<f64>().unwrap();
        let value = BigInt::from_f64(value).unwrap();
        new_int(value)
    }
}

pub fn new_module<S: Into<String>>(name: S, ns: Namespace) -> Arc<RwLock<Module>> {
    Arc::new(RwLock::new(Module::new(name, ns)))
}

pub fn new_str<S: Into<String>>(value: S) -> ObjectRef {
    Arc::new(RwLock::new(Str::new(value)))
}

pub fn new_tuple(items: Vec<ObjectRef>) -> ObjectRef {
    Arc::new(RwLock::new(Tuple::new(items)))
}

// Custom type constructor -----------------------------------------

pub fn new_custom_type(
    module: Arc<RwLock<Module>>,
    name: &str,
) -> Arc<RwLock<CustomType>> {
    Arc::new(RwLock::new(CustomType::new(module, name)))
}

pub fn new_custom_instance(
    class: Arc<RwLock<CustomType>>,
    attrs: Namespace,
) -> ObjectRef {
    Arc::new(RwLock::new(CustomObj::new(class, attrs)))
}

// Utilities -------------------------------------------------------

/// Convert Rust bool to builtin Bool object
pub fn bool_obj_from_bool(value: bool) -> ObjectRef {
    if value {
        new_true()
    } else {
        new_false()
    }
}

/// Collect parameters for function types.
fn collect_params<S: Into<String>>(params: Option<Vec<S>>) -> Params {
    if let Some(names) = params {
        let mut params = vec![];
        for name in names {
            params.push(name.into());
        }
        Some(params)
    } else {
        None
    }
}
