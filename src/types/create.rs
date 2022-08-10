//! Type Constructors.
//!
//! These constructors simplify the creation system objects.
use std::sync::{Arc, RwLock};

use num_bigint::BigInt;
use num_traits::{FromPrimitive, Num, Signed, ToPrimitive, Zero};

use once_cell::sync::Lazy;

use crate::vm::Code;

use super::base::ObjectRef;
use super::bool::Bool;
use super::bound_func::BoundFunc;
use super::builtin_func::{BuiltinFn, BuiltinFunc};
use super::closure::Closure;
use super::custom::{CustomObj, CustomType};
use super::float::Float;
use super::func::Func;
use super::int::Int;
use super::list::List;
use super::module::Module;
use super::nil::Nil;
use super::ns::Namespace;
use super::str::Str;
use super::tuple::Tuple;

use super::result::Params;

static NIL: Lazy<Arc<RwLock<Nil>>> = Lazy::new(|| Arc::new(RwLock::new(Nil::new())));

static TRUE: Lazy<Arc<RwLock<Bool>>> =
    Lazy::new(|| Arc::new(RwLock::new(Bool::new(true))));

static FALSE: Lazy<Arc<RwLock<Bool>>> =
    Lazy::new(|| Arc::new(RwLock::new(Bool::new(false))));

pub static GLOBAL_INT_MAX: Lazy<BigInt> = Lazy::new(|| BigInt::from(256));

pub static SHARED_INTS: Lazy<Vec<Arc<RwLock<Int>>>> = Lazy::new(|| {
    let end = GLOBAL_INT_MAX.to_u32().unwrap();
    (0..=end).map(|i| Arc::new(RwLock::new(Int::new(BigInt::from(i))))).collect()
});

pub fn in_shared_int_range(value: &BigInt) -> bool {
    value.is_zero() || (value.is_positive() && value <= Lazy::force(&GLOBAL_INT_MAX))
}

// Builtin type constructors ---------------------------------------

#[inline]
pub fn new_nil() -> ObjectRef {
    NIL.clone()
}

#[inline]
pub fn new_bool(val: bool) -> ObjectRef {
    if val {
        new_true()
    } else {
        new_false()
    }
}

#[inline]
pub fn new_true() -> ObjectRef {
    TRUE.clone()
}

#[inline]
pub fn new_false() -> ObjectRef {
    FALSE.clone()
}

pub fn new_bound_func(func: ObjectRef, this: ObjectRef) -> ObjectRef {
    Arc::new(RwLock::new(BoundFunc::new(func, this)))
}

pub fn new_builtin_func<S: Into<String>>(
    name: S,
    params: Vec<S>,
    func: BuiltinFn,
) -> ObjectRef {
    let params = collect_params(params);
    Arc::new(RwLock::new(BuiltinFunc::new(name, params, func)))
}

pub fn new_closure(func: ObjectRef) -> ObjectRef {
    Arc::new(RwLock::new(Closure::new(func)))
}

pub fn new_float(value: f64) -> ObjectRef {
    Arc::new(RwLock::new(Float::new(value)))
}

pub fn new_float_from_string<S: Into<String>>(value: S) -> ObjectRef {
    let value = value.into();
    let value = value.parse::<f64>().unwrap();
    new_float(value)
}

pub fn new_func<S: Into<String>>(name: S, params: Vec<S>, code: Code) -> ObjectRef {
    let params = collect_params(params);
    Arc::new(RwLock::new(Func::new(name, params, code)))
}

pub fn new_int<I: Into<BigInt>>(value: I) -> ObjectRef {
    let value = value.into();
    if value.is_positive() && &value <= Lazy::force(&GLOBAL_INT_MAX) {
        let index = value.to_usize().unwrap();
        SHARED_INTS[index].clone()
    } else {
        Arc::new(RwLock::new(Int::new(value)))
    }
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

pub fn new_list(items: Vec<ObjectRef>) -> ObjectRef {
    Arc::new(RwLock::new(List::new(items)))
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

#[allow(dead_code)]
pub fn new_custom_type(
    module: Arc<RwLock<Module>>,
    name: &str,
) -> Arc<RwLock<CustomType>> {
    Arc::new(RwLock::new(CustomType::new(module, name)))
}

#[allow(dead_code)]
pub fn new_custom_instance(
    class: Arc<RwLock<CustomType>>,
    attrs: Namespace,
) -> ObjectRef {
    Arc::new(RwLock::new(CustomObj::new(class, attrs)))
}

// Utilities -------------------------------------------------------

/// Collect parameters for function types.
fn collect_params<S: Into<String>>(params: Vec<S>) -> Params {
    params.into_iter().map(|n| n.into()).collect()
}
