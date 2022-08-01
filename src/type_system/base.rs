//! Type System
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use once_cell::sync::Lazy;

use super::bool::{Bool, BoolType, BOOL_TYPE};
use super::builtins::BUILTINS;
use super::class::{Type, TypeType, TYPE_TYPE};
use super::int::{Int, IntType, INT_TYPE};
use super::module::{Module, ModuleType, MODULE_TYPE};
use super::nil::{Nil, NilType, NIL_TYPE};
use super::ns::{Namespace, NamespaceType, NS_TYPE};
use super::str::{Str, StrType, STR_TYPE};

use super::create;

pub type TypeRef = Arc<dyn TypeTrait>;
pub type ObjectRef = Arc<dyn ObjectTrait>;

// Type Trait ----------------------------------------------------------
//
// A type trait is a container for type attributes such as methods that
// are shared between instances.

pub trait TypeTrait {
    fn module(&self) -> ObjectRef {
        BUILTINS.clone()
    }
    fn name(&self) -> &str;
    fn full_name(&self) -> &str;
    fn namespace(&self) -> ObjectRef;
}

// Object Trait --------------------------------------------------------
//
// The object trait is a container for instance attributes that are NOT
// shared between instances.

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

pub trait ObjectTrait {
    fn as_any(&self) -> &dyn Any;
    fn metaclass(&self) -> TypeRef;
    fn class(&self) -> ObjectRef;
    fn namespace(&self) -> ObjectRef;

    // fn module(&self) -> ObjectRef {
    //     self.metaclass().module().clone()
    // }

    fn id(&self) -> usize {
        let p = self as *const Self;
        let p = p as *const () as usize;
        p
    }

    fn id_obj(&self) -> ObjectRef {
        create::new_int_from_usize(self.id())
    }

    fn get_attr(&self, name: &str) -> Option<ObjectRef> {
        if name == "$type" {
            return Some(self.class().clone());
        }
        if name == "$module" {
            return Some(self.metaclass().module().clone());
        }
        if name == "$id" {
            return Some(self.id_obj());
        }
        let ns = self.namespace();
        if let Some(obj) = ns.to_namespace().unwrap().get_obj(name) {
            return Some(obj);
        }
        let ns = self.class().namespace();
        ns.to_namespace().unwrap().get_obj(name)
    }

    to_type!(to_type, TypeType);
    to_type!(to_bool, Bool);
    to_type!(to_int, Int);
    to_type!(to_module, Module);
    to_type!(to_namespace, Namespace);
    to_type!(to_nil, Nil);
    to_type!(to_str, Str);
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

impl fmt::Display for dyn TypeTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}.{}>", self.module(), self.name())
    }
}

impl fmt::Debug for dyn TypeTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for dyn ObjectTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_instance!(f, self, Type, Bool, Int, Module, Namespace, Nil, Str);
        // Fallback
        write!(f, "{} object @ {}", self.class(), self.id())
    }
}

impl fmt::Debug for dyn ObjectTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_instance!(f, self, Type, Bool, Int, Module, Namespace, Nil, Str);
        // Fallback
        write!(f, "{self}")
    }
}
