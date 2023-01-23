//! Type Constructors.
//!
//! These constructors simplify the creation of system objects.
use std::sync::{Arc, RwLock};

use num_bigint::BigInt;
use num_traits::{FromPrimitive, Num, Signed, ToPrimitive};

use indexmap::IndexMap;
use once_cell::sync::Lazy;

use crate::util::format_doc;
use crate::vm::{globals, Code};

use super::base::ObjectRef;
use super::gen::{obj_ref, obj_ref_t};
use super::result::Params;

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
use super::iterator::FIIterator;
use super::list::List;
use super::map::Map;
use super::module::Module;
use super::ns::Namespace;
use super::prop::Prop;
use super::str::Str;
use super::tuple::Tuple;

// Global singletons ---------------------------------------------------

#[inline]
pub fn nil() -> ObjectRef {
    globals::NIL.clone()
}

#[inline]
pub fn bool(val: bool) -> ObjectRef {
    if val {
        globals::TRUE.clone()
    } else {
        globals::FALSE.clone()
    }
}

#[inline]
pub fn empty_str() -> ObjectRef {
    globals::EMPTY_STR.clone()
}

#[inline]
pub fn empty_tuple() -> ObjectRef {
    globals::EMPTY_TUPLE.clone()
}

// Builtin type constructors -------------------------------------------

pub fn bound_func(func: ObjectRef, this: ObjectRef) -> ObjectRef {
    obj_ref!(BoundFunc::new(func, this))
}

pub fn builtin_func(
    module_name: &str,
    name: &str,
    this_type: Option<ObjectRef>,
    params: &[&str],
    doc: &str,
    func: BuiltinFn,
) -> ObjectRef {
    let params = params.iter().map(|n| n.to_string()).collect();
    let doc = format_doc(doc);
    obj_ref!(BuiltinFunc::new(
        module_name.to_owned(),
        name.to_owned(),
        this_type,
        params,
        str(doc),
        func
    ))
}

pub fn builtin_module(
    name: &str,
    path: &str,
    ns: Namespace,
    doc: &str,
) -> obj_ref_t!(Module) {
    obj_ref!(Module::new(name.into(), path.into(), ns, Code::new(), Some(doc)))
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

pub fn type_err<S: Into<String>>(msg: S, obj: ObjectRef) -> ObjectRef {
    err(ErrKind::Type, msg, obj)
}

static OK_ERR: Lazy<obj_ref_t!(ErrObj)> = Lazy::new(|| {
    obj_ref!(ErrObj::with_responds_to_bool(ErrKind::Ok, "".to_string(), nil()))
});

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

pub fn func<S: Into<String>>(
    module_name: S,
    func_name: S,
    params: Params,
    code: Code,
) -> ObjectRef {
    obj_ref!(Func::new(module_name.into(), func_name.into(), params, code))
}

pub fn int<I: Into<BigInt>>(value: I) -> ObjectRef {
    let value = value.into();
    if value.is_positive() && &value <= Lazy::force(&globals::SHARED_INT_MAX_BIGINT) {
        let index = value.to_usize().unwrap();
        globals::SHARED_INTS[index].clone()
    } else {
        obj_ref!(Int::new(value))
    }
}

pub fn int_from_string<S: Into<String>>(val: S) -> ObjectRef {
    let val = val.into();
    if let Ok(val) = BigInt::from_str_radix(val.as_ref(), 10) {
        int(val)
    } else if let Ok(val) = val.parse::<f64>() {
        int(BigInt::from_f64(val).unwrap())
    } else {
        type_err("Could not convert string to Int", str(val))
    }
}

pub fn iterator(wrapped: Vec<ObjectRef>) -> ObjectRef {
    obj_ref!(FIIterator::new(wrapped))
}

pub fn list(items: Vec<ObjectRef>) -> ObjectRef {
    obj_ref!(List::new(items.to_vec()))
}

pub fn map(entries: Vec<(String, ObjectRef)>) -> ObjectRef {
    let entries: IndexMap<String, ObjectRef> = entries.into_iter().collect();
    obj_ref!(Map::new(entries))
}

pub fn prop(getter: ObjectRef) -> ObjectRef {
    obj_ref!(Prop::new(getter))
}

pub fn str<S: Into<String>>(val: S) -> ObjectRef {
    let val = val.into();
    if val.is_empty() {
        globals::EMPTY_STR.clone()
    } else if val == "\n" {
        globals::NEWLINE.clone()
    } else {
        obj_ref!(Str::new(val))
    }
}

pub fn tuple(items: Vec<ObjectRef>) -> ObjectRef {
    if items.is_empty() {
        globals::EMPTY_TUPLE.clone()
    } else {
        obj_ref!(Tuple::new(items))
    }
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
