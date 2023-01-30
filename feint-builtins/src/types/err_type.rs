//! Error Types
//!
//! Builtin type used to tag builtin `Err` instances.
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::new;
use feint_code_gen::*;

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
    NotCallable,
    String,
    Type,
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
        NotCallable,
        String,
        Type,
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
            NotCallable => "not_callable",
            String => "string",
            Type => "type",
            Ok => "ok",
        }
    }

    pub fn get_obj(&self) -> Option<ObjectRef> {
        let err_type_type = ERR_TYPE_TYPE.read().unwrap();
        err_type_type.ns.get(self.name())
    }
}

// ErrType Type --------------------------------------------------------

type_and_impls!(ErrTypeType, ErrType);

pub static ERR_TYPE_TYPE: Lazy<obj_ref_t!(ErrTypeType)> = Lazy::new(|| {
    let type_ref = obj_ref!(ErrTypeType::new());
    let mut type_obj = type_ref.write().unwrap();

    // Types as class attributes
    for kind in ERR_KINDS.iter() {
        type_obj.add_attr(kind.name(), obj_ref!(ErrTypeObj::new(kind.clone())));
    }

    type_obj.add_attrs(&[
        // Instance Attributes -----------------------------------------
        prop!("name", type_ref, "", |this, _| {
            let this = this.read().unwrap();
            let this = this.as_any().downcast_ref::<ErrTypeObj>().unwrap();
            new::str(this.name())
        }),
    ]);

    type_ref.clone()
});

// ErrType Object ------------------------------------------------------

pub struct ErrTypeObj {
    ns: Namespace,
    kind: ErrKind,
}

standard_object_impls!(ErrTypeObj);

impl ErrTypeObj {
    pub fn new(kind: ErrKind) -> Self {
        Self { ns: Namespace::default(), kind }
    }

    pub fn kind(&self) -> &ErrKind {
        &self.kind
    }

    pub fn name(&self) -> &str {
        self.kind.name()
    }
}

impl ObjectTrait for ErrTypeObj {
    object_trait_header!(ERR_TYPE_TYPE);

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if self.is(rhs) || rhs.is_always() {
            true
        } else if let Some(rhs) = rhs.down_to_err_type_obj() {
            self.kind == rhs.kind
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
            NotCallable => "Not callable",
            String => "String error",
            Type => "Type error",
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
