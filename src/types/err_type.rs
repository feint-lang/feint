//! Error Types
//!
//! This is a builtin type used to tag builtin `Err` instances.
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

#[derive(PartialEq)]
pub enum ErrKind {
    Arg,
    Assertion,
    AttrNotFound,
    NameNotFound,
    Ok,
}

impl ErrKind {
    pub fn from_name(name: &str) -> Option<Self> {
        use ErrKind::*;
        let kind = match name {
            "arg" => Arg,
            "assertion" => Assertion,
            "attr_not_found" => AttrNotFound,
            "name_not_found" => NameNotFound,
            "ok" => Ok,
            _ => return None,
        };
        Some(kind)
    }

    pub fn name(&self) -> &str {
        use ErrKind::*;
        match self {
            Arg => "arg",
            Assertion => "assertion",
            AttrNotFound => "attr_not_found",
            NameNotFound => "name_not_found",
            Ok => "ok",
        }
    }

    pub fn get_obj(&self) -> Option<ObjectRef> {
        let err_type_type = ERR_TYPE_TYPE.read().unwrap();
        err_type_type.ns.get_obj(self.name()).clone()
    }
}

// ErrType Type --------------------------------------------------------

gen::type_and_impls!(ErrTypeType, ErrType);

pub static ERR_TYPE_TYPE: Lazy<new::obj_ref_t!(ErrTypeType)> = Lazy::new(|| {
    let type_ref = new::obj_ref!(ErrTypeType::new());
    let mut class = type_ref.write().unwrap();
    let ns = class.ns_mut();

    ns.add_entries(&[
        // Class Attributes --------------------------------------------
        ("arg", new::obj_ref!(ErrTypeObj::new("arg"))),
        ("assertion", new::obj_ref!(ErrTypeObj::new("assertion"))),
        ("attr_not_found", new::obj_ref!(ErrTypeObj::new("attr_not_found"))),
        ("name_not_found", new::obj_ref!(ErrTypeObj::new("name_not_found"))),
        ("ok", new::obj_ref!(ErrTypeObj::new("ok"))),
        // Instance Attributes -----------------------------------------
        gen::prop!("name", type_ref, |this, _, _| {
            let this = this.read().unwrap();
            let this = this.as_any().downcast_ref::<ErrTypeObj>().unwrap();
            Ok(new::str(&this.name))
        }),
    ]);

    type_ref.clone()
});

// ErrType Object ------------------------------------------------------

pub struct ErrTypeObj {
    ns: Namespace,
    pub name: String,
}

gen::standard_object_impls!(ErrTypeObj);

impl ErrTypeObj {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { ns: Namespace::new(), name: name.into() }
    }

    pub fn kind(&self) -> Option<ErrKind> {
        ErrKind::from_name(self.name.as_str())
    }
}

impl ObjectTrait for ErrTypeObj {
    gen::object_trait_header!(ERR_TYPE_TYPE);

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if let Some(rhs) = rhs.down_to_err_type_obj() {
            self.is(rhs) || self.kind() == rhs.kind()
        } else {
            false
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for ErrKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ErrKind::*;
        let message = match self {
            Arg => "Invalid arg",
            Assertion => "Assertion failed",
            AttrNotFound => "Attribute not found",
            NameNotFound => "Name not found",
            Ok => "OK (not an error)",
        };
        write!(f, "{message}")
    }
}

impl fmt::Debug for ErrKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for ErrTypeObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Debug for ErrTypeObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
