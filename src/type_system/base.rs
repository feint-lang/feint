//! Type System
use std::any::Any;
use std::fmt;
use std::sync::Arc;

use num_bigint::BigInt;

use crate::vm::{RuntimeBoolResult, RuntimeContext, RuntimeErr, RuntimeObjResult, VM};

use super::builtins::BUILTINS;
use super::create;
use super::result::{Args, CallResult};

use super::bool::Bool;
use super::builtin_func::BuiltinFunc;
use super::class::Type;
use super::float::Float;
use super::func::Func;
use super::int::Int;
use super::module::Module;
use super::nil::Nil;
use super::ns::Namespace;
use super::str::Str;
use super::tuple::Tuple;

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

/// Create associated function to check is object ref a specific impl
/// type.
macro_rules! make_type_checker {
    ( $func:ident, $ty:ty) => {
        fn $func(&self) -> bool {
            self.as_any().downcast_ref::<$ty>().is_some()
        }
    };
}

/// Create associated function to downcast from object ref to impl.
macro_rules! make_down_to {
    ( $func:ident, $ty:ty) => {
        fn $func(&self) -> Option<&$ty> {
            self.as_any().downcast_ref::<$ty>()
        }
    };
}

/// Create associated function to extract value from object. This is
/// used only for types that have a simple inner value that's exposed
/// through a `value()` method.
macro_rules! make_value_extractor {
    ( $func:ident, $ty:ty, $val_ty:ty, $op:ident) => {
        fn $func(&self) -> Option<$val_ty> {
            self.as_any().downcast_ref::<$ty>().map(|obj| obj.value().$op())
        }
    };
}

/// Create associated unary op function.
macro_rules! make_unary_op {
    ( $meth:ident, $op:literal, $result:ty ) => {
        fn $meth(&self, _ctx: &RuntimeContext) -> $result {
            Err(RuntimeErr::new_type_err(format!(
                "Unary operator {} ({}) not implemented for {}",
                $op,
                stringify!($meth),
                self.type_obj()
            )))
        }
    };
}

/// Create associated binary op function.
macro_rules! make_bin_op {
    ( $func:ident, $op:literal, $result:ty ) => {
        fn $func(&self, _rhs: &dyn ObjectTrait, _ctx: &RuntimeContext) -> $result {
            Err(RuntimeErr::new_type_err(format!(
                "Binary operator {} ({}) not implemented for {}",
                $op,
                stringify!($func),
                self.type_obj()
            )))
        }
    };
}

/// Objects in the system--instances of types--are backed by an
/// implementation of `ObjectTrait`. Example: `Int`.
pub trait ObjectTrait {
    fn as_any(&self) -> &dyn Any;

    /// Get an instance's type as a type. This is needed to retrieve
    /// type level attributes.
    fn class(&self) -> TypeRef;

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

    // Attributes (accessed by name) -----------------------------------

    fn get_attr(&self, name: &str) -> Option<ObjectRef> {
        if name == "$type" {
            return Some(self.type_obj().clone());
        }
        if name == "$module" {
            return Some(self.class().module().clone());
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

    // Items (accessed by index) ---------------------------------------

    fn get_item(&self, _index: usize) -> Option<ObjectRef> {
        None
    }

    // Type checkers ---------------------------------------------------

    make_type_checker!(is_bool, Bool);
    make_type_checker!(is_builtin_func, BuiltinFunc);
    make_type_checker!(is_float, Float);
    make_type_checker!(is_func, Func);
    make_type_checker!(is_int, Int);
    make_type_checker!(is_nil, Nil);
    make_type_checker!(is_str, Str);
    make_type_checker!(is_tuple, Tuple);

    // Downcasters -----------------------------------------------------
    //
    // These downcast object refs to their concrete types.

    make_down_to!(down_to_type, Type);
    make_down_to!(down_to_bool, Bool);
    make_down_to!(down_to_float, Float);
    make_down_to!(down_to_int, Int);
    make_down_to!(down_to_mod, Module);
    make_down_to!(down_to_ns, Namespace);
    make_down_to!(down_to_nil, Nil);
    make_down_to!(down_to_str, Str);
    make_down_to!(down_to_tuple, Tuple);

    // Value extractors ------------------------------------------------
    //
    // These extract the inner value from an object.

    make_value_extractor!(get_float_val, Float, f64, clone);
    make_value_extractor!(get_int_val, Int, BigInt, clone);
    make_value_extractor!(get_str_val, Str, String, to_owned);

    // Unary operations ------------------------------------------------

    make_unary_op!(negate, "-", RuntimeObjResult);
    make_unary_op!(bool_val, "!!", RuntimeBoolResult);

    fn not(&self, ctx: &RuntimeContext) -> RuntimeBoolResult {
        match self.bool_val(ctx) {
            Ok(true) => Ok(false),
            Ok(false) => Ok(true),
            err => err,
        }
    }

    // Binary operations -----------------------------------------------

    fn is_equal(&self, rhs: &dyn ObjectTrait, _ctx: &RuntimeContext) -> bool {
        // This duplicates ObjectExt::is(), but that method can't be
        // used here.
        self.id() == rhs.id()
    }

    fn not_equal(&self, rhs: &dyn ObjectTrait, ctx: &RuntimeContext) -> bool {
        !self.is_equal(rhs, ctx)
    }

    make_bin_op!(less_than, "<", RuntimeBoolResult);
    make_bin_op!(greater_than, ">", RuntimeBoolResult);

    make_bin_op!(pow, "^", RuntimeObjResult);
    make_bin_op!(modulo, "%", RuntimeObjResult);
    make_bin_op!(mul, "*", RuntimeObjResult);
    make_bin_op!(div, "/", RuntimeObjResult);
    make_bin_op!(floor_div, "//", RuntimeObjResult);
    make_bin_op!(add, "+", RuntimeObjResult);
    make_bin_op!(sub, "-", RuntimeObjResult);
    make_bin_op!(and, "&&", RuntimeBoolResult);
    make_bin_op!(or, "||", RuntimeBoolResult);

    // Call ------------------------------------------------------------

    fn call(&self, _args: Args, _vm: &mut VM) -> CallResult {
        let class = self.class();
        Err(RuntimeErr::new_type_err(format!("Call not implemented for type {class}")))
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
