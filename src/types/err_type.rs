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

#[derive(Clone, Debug, PartialEq)]
pub enum ErrKind {
    Arg,
    Assertion,
    Attr,         // generic attribute error
    AttrNotFound, // more specific attribute not found error
    FileNotFound,
    FileUnreadable,
    IndexOutOfBounds,
    String,
    Ok,
}

static ERR_KINDS: Lazy<Vec<ErrKind>> = Lazy::new(|| {
    use ErrKind::*;
    vec![
        Arg,
        Assertion,
        Attr,
        AttrNotFound,
        FileNotFound,
        FileUnreadable,
        IndexOutOfBounds,
        String,
        Ok,
    ]
});

impl ErrKind {
    pub fn name(&self) -> &str {
        use ErrKind::*;
        match self {
            Arg => "arg",
            Assertion => "assertion",
            Attr => "attr",
            AttrNotFound => "attr_not_found",
            FileNotFound => "file_not_found",
            FileUnreadable => "file_unreadable",
            IndexOutOfBounds => "index_out_of_bounds",
            String => "string",
            Ok => "ok",
        }
    }

    pub fn get_obj(&self) -> Option<ObjectRef> {
        let err_type_type = ERR_TYPE_TYPE.read().unwrap();
        err_type_type.ns.get_obj(self.name())
    }
}

// ErrType Type --------------------------------------------------------

gen::type_and_impls!(ErrTypeType, ErrType);

pub static ERR_TYPE_TYPE: Lazy<new::obj_ref_t!(ErrTypeType)> = Lazy::new(|| {
    let type_ref = new::obj_ref!(ErrTypeType::new());
    let mut class = type_ref.write().unwrap();
    let ns = class.ns_mut();

    // Types as class attributes
    for kind in ERR_KINDS.iter() {
        ns.add_entry((kind.name(), new::obj_ref!(ErrTypeObj::new(kind.clone()))));
    }

    ns.add_entries(&[
        // Instance Attributes -----------------------------------------
        gen::prop!("name", type_ref, |this, _, _| {
            let this = this.read().unwrap();
            let this = this.as_any().downcast_ref::<ErrTypeObj>().unwrap();
            Ok(new::str(this.name()))
        }),
    ]);

    type_ref.clone()
});

// ErrType Object ------------------------------------------------------

pub struct ErrTypeObj {
    ns: Namespace,
    kind: ErrKind,
}

gen::standard_object_impls!(ErrTypeObj);

impl ErrTypeObj {
    pub fn new(kind: ErrKind) -> Self {
        Self { ns: Namespace::new(), kind }
    }

    pub fn kind(&self) -> &ErrKind {
        &self.kind
    }

    pub fn name(&self) -> &str {
        self.kind.name()
    }
}

impl ObjectTrait for ErrTypeObj {
    gen::object_trait_header!(ERR_TYPE_TYPE);

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if let Some(rhs) = rhs.down_to_err_type_obj() {
            self.is(rhs) || self.kind == rhs.kind
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
            Attr => "Attribute error",
            AttrNotFound => "Attribute not found",
            FileNotFound => "File not found",
            FileUnreadable => "File could not be read",
            IndexOutOfBounds => "Index out of bounds",
            String => "String error",
            Ok => "OK (not an error)",
        };
        write!(f, "{message}")
    }
}

impl fmt::Display for ErrTypeObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Debug for ErrTypeObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ErrType.{}", self.name())
    }
}
