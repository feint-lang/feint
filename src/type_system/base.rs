//! Type System
use crate::type_system::create;
use std::any::Any;
use std::fmt;
use std::sync::Arc;

use super::bool::{Bool, BoolType};
use super::class::{Type, TypeType};
use super::int::{Int, IntType};
use super::nil::{Nil, NilType};
use super::ns::{Namespace, NamespaceType};
use super::str::{Str, StrType};

// Macros --------------------------------------------------------------

macro_rules! to_type {
    ( $func:ident, $ty:ty) => {
        fn $func(&self) -> Option<&$ty> {
            if let Some(obj) = self.as_any().downcast_ref::<$ty>() {
                Some(obj)
            } else {
                None
            }
        }
    };
}

pub type ObjectRef = Arc<dyn ObjectTrait>;

// Type Trait ----------------------------------------------------------
//
// A type trait is a container for type attributes such as methods that
// are shared between instances.

pub trait TypeTrait {
    fn module(&self) -> &str {
        "builtins"
    }
    fn name(&self) -> &str;
    fn full_name(&self) -> &str;
}

// Object Trait --------------------------------------------------------
//
// The object trait is a container for instance attributes that are NOT
// shared between instances.

pub trait ObjectTrait {
    fn as_any(&self) -> &dyn Any;
    fn class(&self) -> ObjectRef;
    fn namespace(&self) -> ObjectRef;

    fn id(&self) -> usize {
        let p = self as *const Self;
        let p = p as *const () as usize;
        p
    }

    fn id_obj(&self) -> ObjectRef {
        create::new_int_from_usize(self.id())
    }

    to_type!(to_type, Type);
    to_type!(to_bool, Bool);
    to_type!(to_int, Int);
    to_type!(to_namespace, Namespace);
    to_type!(to_nil, Nil);
    to_type!(to_str, Str);

    fn get_attr(&self, name: &str) -> Option<ObjectRef> {
        if name == "$type" {
            return Some(self.class().clone());
        }
        if name == "$id" {
            return Some(self.id_obj());
        }
        if let Some(obj) = self.namespace().to_namespace().unwrap().get_obj(name) {
            return Some(obj);
        }
        self.class().namespace().to_namespace().unwrap().get_obj(name)
    }
}

pub trait ObjectTraitExt: ObjectTrait {
    fn is(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T: ObjectTrait + ?Sized> ObjectTraitExt for T {}

// Display -------------------------------------------------------------

macro_rules! write_instance {
    ( $f:ident, $a:ident, $($A:ty),+ ) => { $(
        if let Some(a) = $a.as_any().downcast_ref::<$A>() {
            return write!($f, "{}", a);
        }
    )+ };
}

macro_rules! debug_instance {
    ( $f:ident, $a:ident, $($A:ty),+ ) => { $(
        if let Some(a) = $a.as_any().downcast_ref::<$A>() {
            return write!($f, "{:?}", a);
        }
    )+ };
}

impl fmt::Display for dyn ObjectTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_instance!(
            f,
            self,
            TypeType,
            Type,
            BoolType,
            Bool,
            IntType,
            Int,
            NamespaceType,
            Namespace,
            NilType,
            Nil,
            StrType,
            Str
        );
        // Fallback
        write!(f, "{} object @ {}", self.class(), self.id())
    }
}

impl fmt::Debug for dyn ObjectTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_instance!(
            f,
            self,
            TypeType,
            Type,
            BoolType,
            Bool,
            IntType,
            Int,
            NamespaceType,
            Namespace,
            NilType,
            Nil,
            StrType,
            Str
        );
        // Fallback
        write!(f, "{self}")
    }
}
