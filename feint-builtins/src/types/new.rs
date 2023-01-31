//! Type Constructors.
//!
//! These constructors simplify the creation of system objects.
use std::sync::{Arc, RwLock};

use num_bigint::BigInt;
use num_traits::{FromPrimitive, Num, Signed, ToPrimitive};

use indexmap::IndexMap;
use once_cell::sync::Lazy;

use feint_code_gen::{obj_ref, obj_ref_t};
use feint_util::string::format_doc;

use super::base::{ObjectRef, ObjectTrait};
use super::Params;

use super::always::Always;
use super::bool::Bool;
use super::bound_func::BoundFunc;
use super::cell::Cell;
use super::closure::Closure;
use super::code::Code;
use super::custom::{CustomObj, CustomType};
use super::err::ErrObj;
use super::err_type::ErrKind;
use super::file::File;
use super::float::Float;
use super::func::Func;
use super::int::Int;
use super::intrinsic_func::{IntrinsicFn, IntrinsicFunc};
use super::iterator::FIIterator;
use super::list::List;
use super::map::Map;
use super::module::Module;
use super::nil::Nil;
use super::ns::Namespace;
use super::prop::Prop;
use super::str::Str;
use super::tuple::Tuple;

// Global singletons ---------------------------------------------------

static NIL: Lazy<obj_ref_t!(Nil)> = Lazy::new(|| obj_ref!(Nil::new()));
static TRUE: Lazy<obj_ref_t!(Bool)> = Lazy::new(|| obj_ref!(Bool::new(true)));
static FALSE: Lazy<obj_ref_t!(Bool)> = Lazy::new(|| obj_ref!(Bool::new(false)));
static ALWAYS: Lazy<obj_ref_t!(Always)> = Lazy::new(|| obj_ref!(Always::new()));

static EMPTY_STR: Lazy<obj_ref_t!(Str)> =
    Lazy::new(|| obj_ref!(Str::new("".to_owned())));

static NEWLINE: Lazy<obj_ref_t!(Str)> =
    Lazy::new(|| obj_ref!(Str::new("\n".to_owned())));

static EMPTY_TUPLE: Lazy<obj_ref_t!(Tuple)> =
    Lazy::new(|| obj_ref!(Tuple::new(vec![])));

static SHARED_INT_MAX: usize = 256;
static SHARED_INT_MAX_BIGINT: Lazy<BigInt> = Lazy::new(|| BigInt::from(SHARED_INT_MAX));
static SHARED_INTS: Lazy<Vec<obj_ref_t!(Int)>> = Lazy::new(|| {
    (0..=SHARED_INT_MAX).map(|i| obj_ref!(Int::new(BigInt::from(i)))).collect()
});

#[inline]
pub fn nil() -> ObjectRef {
    NIL.clone()
}

#[inline]
pub fn bool(val: bool) -> ObjectRef {
    if val {
        TRUE.clone()
    } else {
        FALSE.clone()
    }
}

#[inline]
pub fn always() -> ObjectRef {
    ALWAYS.clone()
}

#[inline]
pub fn empty_str() -> ObjectRef {
    EMPTY_STR.clone()
}

#[inline]
pub fn newline() -> ObjectRef {
    NEWLINE.clone()
}

#[inline]
pub fn empty_tuple() -> ObjectRef {
    EMPTY_TUPLE.clone()
}

// Intrinsic type constructors ---------------------------------

pub fn bound_func(func: ObjectRef, this: ObjectRef) -> ObjectRef {
    obj_ref!(BoundFunc::new(func, this))
}

pub fn intrinsic_func(
    module_name: &str,
    name: &str,
    this_type: Option<ObjectRef>,
    params: &[&str],
    doc: &str,
    func: IntrinsicFn,
) -> ObjectRef {
    let params = params.iter().map(|n| n.to_string()).collect();
    let doc = format_doc(doc);
    obj_ref!(IntrinsicFunc::new(
        module_name.to_owned(),
        name.to_owned(),
        this_type,
        params,
        str(doc),
        func
    ))
}

pub fn intrinsic_module(
    name: &str,
    path: &str,
    doc: &str,
    entries: &[(&str, ObjectRef)],
) -> obj_ref_t!(Module) {
    obj_ref!(Module::with_entries(
        entries,
        name.to_owned(),
        path.to_owned(),
        Code::default(),
        Some(doc.to_owned())
    ))
}

pub fn cell() -> ObjectRef {
    obj_ref!(Cell::new())
}

pub fn cell_with_value(value: ObjectRef) -> ObjectRef {
    obj_ref!(Cell::with_value(value))
}

pub fn closure(func: ObjectRef, captured: ObjectRef) -> ObjectRef {
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

pub fn not_callable_err(obj: ObjectRef) -> ObjectRef {
    err(ErrKind::NotCallable, format!("{}", obj.read().unwrap()), obj)
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
    if value.is_positive() && &value <= Lazy::force(&SHARED_INT_MAX_BIGINT) {
        let index = value.to_usize().unwrap();
        SHARED_INTS[index].clone()
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

pub fn map(map: IndexMap<String, ObjectRef>) -> ObjectRef {
    obj_ref!(Map::new(map))
}

pub fn map_from_keys_and_vals(keys: Vec<String>, vals: Vec<ObjectRef>) -> ObjectRef {
    assert_eq!(keys.len(), vals.len());
    obj_ref!(Map::new(IndexMap::from_iter(keys.into_iter().zip(vals))))
}

pub fn prop(getter: ObjectRef) -> ObjectRef {
    obj_ref!(Prop::new(getter))
}

pub fn str<S: Into<String>>(val: S) -> ObjectRef {
    let val = val.into();
    if val.is_empty() {
        EMPTY_STR.clone()
    } else if val == "\n" {
        NEWLINE.clone()
    } else {
        obj_ref!(Str::new(val))
    }
}

pub fn tuple(items: Vec<ObjectRef>) -> ObjectRef {
    if items.is_empty() {
        EMPTY_TUPLE.clone()
    } else {
        obj_ref!(Tuple::new(items))
    }
}

pub fn argv_tuple(argv: &[String]) -> ObjectRef {
    obj_ref!(Tuple::new(argv.iter().map(str).collect()))
}

// Custom type constructor ---------------------------------------------

pub fn custom_type(module: ObjectRef, name: &str) -> ObjectRef {
    let class_ref = obj_ref!(CustomType::new(module.clone(), name.to_owned()));

    {
        let mut class = class_ref.write().unwrap();
        let ns = class.ns_mut();
        ns.insert(
            "new",
            intrinsic_func(
                module.read().unwrap().down_to_mod().unwrap().name(),
                name,
                Some(class_ref.clone()),
                &["attrs"],
                "Create a new custom type.

                # Args

                - class: TypeRef
                - type_obj: ObjectRef
                - attributes: Map

                ",
                |this, args| {
                    let attrs_arg = args.get(0).unwrap();
                    let attrs_arg = attrs_arg.read().unwrap();
                    let attrs = attrs_arg.down_to_map().unwrap();

                    let mut ns = Namespace::default();
                    ns.extend_from_map(attrs);

                    // XXX: Cloning the inner object is wonky and breaks
                    //      identity testing.
                    let type_obj = this.read().unwrap();
                    let type_obj = if type_obj.is_type_object() {
                        // Called via custom type.
                        let type_obj = type_obj.down_to_custom_type().unwrap();
                        obj_ref!(type_obj.clone())
                    } else {
                        // Called via custom instance.
                        // XXX: This branch isn't reachable because the
                        //      VM will panic due to the identity test
                        //      issue noted above.
                        let type_obj = type_obj.type_obj();
                        let type_obj = type_obj.read().unwrap();
                        let type_obj = type_obj.down_to_custom_type().unwrap();
                        obj_ref!(type_obj.clone())
                    };

                    let instance = CustomObj::new(type_obj, ns);
                    obj_ref!(instance)
                },
            ),
        );
    }

    class_ref
}
