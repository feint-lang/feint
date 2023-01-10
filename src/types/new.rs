//! Type Constructors.
//!
//! These constructors simplify the creation system objects.
use std::sync::{Arc, RwLock};

use num_bigint::BigInt;
use num_traits::{FromPrimitive, Num, Signed, ToPrimitive, Zero};

use indexmap::IndexMap;
use once_cell::sync::Lazy;

use super::result::Params;
use crate::vm::Code;

use super::always::Always;
use super::base::ObjectRef;
use super::bool::Bool;
use super::bound_func::BoundFunc;
use super::builtin_func::{BuiltinFn, BuiltinFunc};
use super::cell::Cell;
use super::closure::Closure;
use super::custom::{CustomObj, CustomType};
use super::err::ErrObj;
use super::err_type::ErrKind;
use super::file::File;
use super::float::Float;
use super::func::Func;
use super::int::Int;
use super::list::List;
use super::map::Map;
use super::module::Module;
use super::nil::Nil;
use super::ns::Namespace;
use super::prop::Prop;
use super::str::Str;
use super::tuple::Tuple;

static NIL: Lazy<obj_ref_t!(Nil)> = Lazy::new(|| obj_ref!(Nil::new()));
static TRUE: Lazy<obj_ref_t!(Bool)> = Lazy::new(|| obj_ref!(Bool::new(true)));
static FALSE: Lazy<obj_ref_t!(Bool)> = Lazy::new(|| obj_ref!(Bool::new(false)));
static ALWAYS: Lazy<obj_ref_t!(Always)> = Lazy::new(|| obj_ref!(Always::new()));
static EMPTY_STR: Lazy<obj_ref_t!(Str)> =
    Lazy::new(|| obj_ref!(Str::new("".to_owned())));
static EMPTY_TUPLE: Lazy<obj_ref_t!(Tuple)> =
    Lazy::new(|| obj_ref!(Tuple::new(vec![])));

static OK_ERR: Lazy<obj_ref_t!(ErrObj)> = Lazy::new(|| {
    obj_ref!(ErrObj::with_responds_to_bool(ErrKind::Ok, "".to_string(), nil()))
});

static SHARED_INT_INDEX: usize = 6;
static SHARED_INT_MAX: usize = 256;
static SHARED_INT_MAX_BIGINT: Lazy<BigInt> = Lazy::new(|| BigInt::from(SHARED_INT_MAX));
pub static SHARED_INTS: Lazy<Vec<obj_ref_t!(Int)>> = Lazy::new(|| {
    (0..=SHARED_INT_MAX).map(|i| obj_ref!(Int::new(BigInt::from(i)))).collect()
});

/// Get the corresponding global constant index for an int, if the int
/// is in the shared int range [0, 256].
pub fn shared_int_global_const_index(int: &BigInt) -> Option<usize> {
    if int.is_zero() {
        Some(SHARED_INT_INDEX)
    } else if int.is_positive() && int <= Lazy::force(&SHARED_INT_MAX_BIGINT) {
        Some(int.to_usize().unwrap() + SHARED_INT_INDEX)
    } else {
        None
    }
}

/// Get the corresponding shared int object for the specified global
/// constant index, if the index is in the shared int range.
pub fn shared_int_for_global_const_index(index: usize) -> Option<ObjectRef> {
    let i = SHARED_INT_INDEX;
    let j = i + SHARED_INT_MAX;
    if (i..=j).contains(&index) {
        Some(SHARED_INTS[index - SHARED_INT_INDEX].clone())
    } else {
        None
    }
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

use crate::util::format_doc;
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

#[inline]
pub fn always() -> ObjectRef {
    ALWAYS.clone()
}

pub fn bound_func(func: ObjectRef, this: ObjectRef) -> ObjectRef {
    obj_ref!(BoundFunc::new(func, this))
}

pub fn builtin_func(
    name: &str,
    this_type: Option<ObjectRef>,
    params: &[&str],
    doc: &str,
    func: BuiltinFn,
) -> ObjectRef {
    let params = params.iter().map(|n| n.to_string()).collect();
    let doc = format_doc(doc);
    obj_ref!(BuiltinFunc::new(name.to_owned(), this_type, params, str(doc), func))
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

// Errors --------------------------------------------------------------

pub fn err<S: Into<String>>(kind: ErrKind, msg: S, obj: ObjectRef) -> ObjectRef {
    obj_ref!(ErrObj::new(kind, msg.into(), obj))
}

pub fn err_with_responds_to_bool<S: Into<String>>(
    kind: ErrKind,
    msg: S,
    obj: ObjectRef,
) -> ObjectRef {
    obj_ref!(ErrObj::with_responds_to_bool(kind, msg.into(), obj))
}

pub fn arg_err<S: Into<String>>(msg: S, obj: ObjectRef) -> ObjectRef {
    err(ErrKind::Arg, msg, obj)
}

pub fn assertion_err<S: Into<String>>(msg: S, obj: ObjectRef) -> ObjectRef {
    err(ErrKind::Assertion, msg, obj)
}

pub fn attr_err<S: Into<String>>(msg: S, obj: ObjectRef) -> ObjectRef {
    err(ErrKind::Attr, msg, obj)
}

pub fn attr_not_found_err<S: Into<String>>(msg: S, obj: ObjectRef) -> ObjectRef {
    err(ErrKind::AttrNotFound, msg, obj)
}

pub fn file_not_found_err<S: Into<String>>(msg: S, obj: ObjectRef) -> ObjectRef {
    err(ErrKind::FileNotFound, msg, obj)
}

pub fn file_unreadable_err<S: Into<String>>(msg: S, obj: ObjectRef) -> ObjectRef {
    err(ErrKind::FileUnreadable, msg, obj)
}

pub fn index_out_of_bounds_err(index: usize, obj: ObjectRef) -> ObjectRef {
    err(ErrKind::IndexOutOfBounds, index.to_string(), obj)
}

pub fn module_not_found_err<S: Into<String>>(msg: S, obj: ObjectRef) -> ObjectRef {
    err(ErrKind::ModuleNotFound, msg, obj)
}

pub fn module_could_not_be_loaded<S: Into<String>>(
    msg: S,
    obj: ObjectRef,
) -> ObjectRef {
    err(ErrKind::ModuleCouldNotBeLoaded, msg, obj)
}

pub fn string_err<S: Into<String>>(msg: S, obj: ObjectRef) -> ObjectRef {
    err(ErrKind::String, msg, obj)
}

pub fn ok_err() -> ObjectRef {
    OK_ERR.clone()
}

// END Errors ----------------------------------------------------------

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
pub fn func<S: Into<String>>(name: S, params: Params, code: Code) -> ObjectRef {
    obj_ref!(Func::new(name.into(), params, code))
}

pub fn int<I: Into<BigInt>>(value: I) -> ObjectRef {
    let value = value.into();
    if value.is_positive() && &value <= Lazy::force(&SHARED_INT_MAX_BIGINT) {
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

pub fn _module<S: Into<String>>(
    name: S,
    ns: Namespace,
    code: Code,
) -> obj_ref_t!(Module) {
    obj_ref!(Module::new(name.into(), ns, code))
}

pub fn prop(getter: ObjectRef) -> ObjectRef {
    obj_ref!(Prop::new(getter))
}

pub fn str<S: Into<String>>(val: S) -> ObjectRef {
    let val = val.into();
    if val.is_empty() {
        empty_str()
    } else {
        obj_ref!(Str::new(val))
    }
}

pub fn empty_str() -> ObjectRef {
    EMPTY_STR.clone()
}

pub fn tuple(items: Vec<ObjectRef>) -> ObjectRef {
    if items.is_empty() {
        EMPTY_TUPLE.clone()
    } else {
        obj_ref!(Tuple::new(items))
    }
}

pub fn empty_tuple() -> ObjectRef {
    EMPTY_TUPLE.clone()
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
