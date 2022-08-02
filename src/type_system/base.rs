//! Type System
use std::any::Any;
use std::fmt;
use std::sync::Arc;

use super::builtins::BUILTINS;
use super::create;

use super::bool::Bool;
use super::class::Type;
use super::int::Int;
use super::module::Module;
use super::nil::Nil;
use super::ns::Namespace;
use super::str::Str;

pub type TypeRef = Arc<dyn TypeTrait>;
pub type ObjectRef = Arc<dyn ObjectTrait>;

// Type Trait ----------------------------------------------------------

/// Types in the system are backed by an implementation of `TypeTrait`.
/// Each type implementation will be instantiated exactly once (i.e.,
/// types are singletons). Example: `IntType`.
pub trait TypeTrait {
    fn module(&self) -> ObjectRef {
        BUILTINS.clone()
    }
    fn name(&self) -> &str;
    fn full_name(&self) -> &str;
}

// Object Trait --------------------------------------------------------

/// Create associated function to downcast from object ref to impl.
macro_rules! make_down_to {
    ( $func:ident, $ty:ty) => {
        fn $func(&self) -> Option<&$ty> {
            self.as_any().downcast_ref::<$ty>()
        }
    };
}

/// Objects in the system--instances of types--are backed by an
/// implementation of `ObjectTrait`. Example: `Int`.
pub trait ObjectTrait {
    fn as_any(&self) -> &dyn Any;

    /// Get an instance's type as a type. This is needed to retrieve
    /// type level attributes.
    fn type_type(&self) -> TypeRef;

    /// Get an instance's type as an object. This is needed so the type
    /// can be used in object contexts.
    fn type_obj(&self) -> ObjectRef;

    /// Each object has a namespace that holds its attributes.
    fn namespace(&self) -> ObjectRef;

    fn id(&self) -> usize {
        let p = self as *const Self;
        p as *const () as usize
    }

    fn id_obj(&self) -> ObjectRef {
        create::new_int_from_usize(self.id())
    }

    fn get_attr(&self, name: &str) -> Option<ObjectRef> {
        if name == "$type" {
            return Some(self.type_obj().clone());
        }
        if name == "$module" {
            return Some(self.type_type().module().clone());
        }
        if name == "$id" {
            return Some(self.id_obj());
        }
        let ns = self.namespace();
        if let Some(obj) = ns.down_to_ns().unwrap().get_obj(name) {
            return Some(obj);
        }
        let ns = self.type_obj().namespace();
        ns.down_to_ns().unwrap().get_obj(name)
    }

    make_down_to!(down_to_type, Type);
    make_down_to!(down_to_bool, Bool);
    make_down_to!(down_to_int, Int);
    make_down_to!(down_to_mod, Module);
    make_down_to!(down_to_ns, Namespace);
    make_down_to!(down_to_nil, Nil);
    make_down_to!(down_to_str, Str);
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
        write!(f, "{} object @ {}", self.type_obj(), self.id())
    }
}

impl fmt::Debug for dyn ObjectTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_instance!(f, self, Type, Bool, Int, Module, Namespace, Nil, Str);
        // Fallback
        write!(f, "{self}")
    }
}
