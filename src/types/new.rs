//! Type Constructors.
//!
//! These constructors simplify the creation system objects.
use std::sync::{Arc, RwLock};

use num_bigint::BigInt;
use num_traits::{FromPrimitive, Num, Signed, ToPrimitive, Zero};

use indexmap::IndexMap;
use once_cell::sync::Lazy;

use crate::vm::Code;

use super::base::ObjectRef;
use super::bool::Bool;
use super::bound_func::BoundFunc;
use super::builtin_func::{BuiltinFn, BuiltinFunc};
use super::cell::Cell;
use super::closure::Closure;
use super::custom::{CustomObj, CustomType};
use super::file::File;
use super::float::Float;
use super::func::Func;
use super::int::Int;
use super::list::List;
use super::map::Map;
use super::module::Module;
use super::nil::Nil;
use super::ns::Namespace;
use super::str::Str;
use super::tuple::Tuple;

use super::result::Params;

static NIL: Lazy<obj_ref_t!(Nil)> = Lazy::new(|| obj_ref!(Nil::new()));

static TRUE: Lazy<obj_ref_t!(Bool)> = Lazy::new(|| obj_ref!(Bool::new(true)));

static FALSE: Lazy<obj_ref_t!(Bool)> = Lazy::new(|| obj_ref!(Bool::new(false)));

pub static GLOBAL_INT_MAX: Lazy<BigInt> = Lazy::new(|| BigInt::from(256));

pub static SHARED_INTS: Lazy<Vec<obj_ref_t!(Int)>> = Lazy::new(|| {
    let end = GLOBAL_INT_MAX.to_u32().unwrap();
    (0..=end).map(|i| obj_ref!(Int::new(BigInt::from(i)))).collect()
});

pub fn in_shared_int_range(value: &BigInt) -> bool {
    value.is_zero() || (value.is_positive() && value <= Lazy::force(&GLOBAL_INT_MAX))
}

// Builtin type constructors -------------------------------------------

macro_rules! obj_ref_t {
    ( $ty:ty ) => {
        Arc<RwLock<$ty>>
    };
}

pub(crate) use obj_ref_t;

macro_rules! obj_ref {
    ( $obj:expr ) => {
        Arc::new(RwLock::new($obj))
    };
}

pub(crate) use obj_ref;

#[inline]
pub fn nil() -> ObjectRef {
    NIL.clone()
}

#[inline]
pub fn bool(val: bool) -> ObjectRef {
    if val {
        true_()
    } else {
        false_()
    }
}

#[inline]
pub fn true_() -> ObjectRef {
    TRUE.clone()
}

#[inline]
pub fn false_() -> ObjectRef {
    FALSE.clone()
}

pub fn bound_func(func: ObjectRef, this: ObjectRef) -> ObjectRef {
    obj_ref!(BoundFunc::new(func, this))
}

pub fn builtin_func(
    name: &str,
    this_type: Option<ObjectRef>,
    params: &[&str],
    func: BuiltinFn,
) -> ObjectRef {
    let params = params.iter().map(|n| n.to_string()).collect();
    obj_ref!(BuiltinFunc::new(name.to_owned(), this_type, params, func))
}

pub fn builtin_module(name: &str, ns: Namespace) -> obj_ref_t!(Module) {
    obj_ref!(Module::new(name.into(), ns, Code::new()))
}

pub fn cell() -> ObjectRef {
    obj_ref!(Cell::new())
}

pub fn cell_with_value(value: ObjectRef) -> ObjectRef {
    obj_ref!(Cell::with_value(value))
}

pub fn closure(func: ObjectRef, captured: IndexMap<String, ObjectRef>) -> ObjectRef {
    obj_ref!(Closure::new(func, captured))
}

pub fn file<S: Into<String>>(file_name: S) -> ObjectRef {
    obj_ref!(File::new(file_name.into()))
}

pub fn float(value: f64) -> ObjectRef {
    obj_ref!(Float::new(value))
}

pub fn float_from_string<S: Into<String>>(value: S) -> ObjectRef {
    let value = value.into();
    let value = value.parse::<f64>().unwrap();
    float(value)
}

/// NOTE: User functions are created in the compiler where name and
///       params are already owned, so we don't do any conversion here
///       like with builtin functions above.
pub fn func(name: String, params: Params, code: Code) -> ObjectRef {
    obj_ref!(Func::new(name, params, code))
}

pub fn int<I: Into<BigInt>>(value: I) -> ObjectRef {
    let value = value.into();
    if value.is_positive() && &value <= Lazy::force(&GLOBAL_INT_MAX) {
        let index = value.to_usize().unwrap();
        SHARED_INTS[index].clone()
    } else {
        obj_ref!(Int::new(value))
    }
}

pub fn int_from_string<S: Into<String>>(value: S) -> ObjectRef {
    let value = value.into();
    if let Ok(value) = BigInt::from_str_radix(value.as_ref(), 10) {
        int(value)
    } else {
        let value = value.parse::<f64>().unwrap();
        let value = BigInt::from_f64(value).unwrap();
        int(value)
    }
}

pub fn list(items: Vec<ObjectRef>) -> ObjectRef {
    obj_ref!(List::new(items.to_vec()))
}

pub fn map(entries: Vec<(String, ObjectRef)>) -> ObjectRef {
    let entries: IndexMap<String, ObjectRef> = entries.into_iter().collect();
    obj_ref!(Map::new(entries))
}

pub fn module<S: Into<String>>(
    name: S,
    ns: Namespace,
    code: Code,
) -> obj_ref_t!(Module) {
    obj_ref!(Module::new(name.into(), ns, code))
}

pub fn str<S: Into<String>>(value: S) -> ObjectRef {
    obj_ref!(Str::new(value.into()))
}

pub fn tuple(items: Vec<ObjectRef>) -> ObjectRef {
    obj_ref!(Tuple::new(items))
}

// Custom type constructor ---------------------------------------------

#[allow(dead_code)]
pub fn custom_type(module: obj_ref_t!(Module), name: &str) -> obj_ref_t!(CustomType) {
    obj_ref!(CustomType::new(module, name.into()))
}

#[allow(dead_code)]
pub fn custom_instance(class: obj_ref_t!(CustomType), attrs: Namespace) -> ObjectRef {
    obj_ref!(CustomObj::new(class, attrs))
}
