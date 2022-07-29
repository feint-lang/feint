use std::any::Any;
use std::fmt;
use std::sync::Arc;

use num_bigint::BigInt;

use crate::vm::{RuntimeBoolResult, RuntimeContext, RuntimeErr, RuntimeObjResult, VM};

use super::result::{Args, CallResult, GetAttrResult, SetAttrResult};

use super::bool::Bool;
use super::builtin_func::BuiltinFunc;
use super::class::{Type, TypeRef};
use super::custom::Custom;
use super::float::Float;
use super::func::Func;
use super::int::Int;
use super::namespace::Namespace;
use super::nil::Nil;
use super::str::Str;
use super::tuple::Tuple;

pub type ObjectRef = Arc<dyn Object>;

macro_rules! make_type_checker {
    ( $func:ident, $ty:ty) => {
        fn $func(&self) -> bool {
            match self.as_any().downcast_ref::<$ty>() {
                Some(_) => true,
                None => false,
            }
        }
    };
}

macro_rules! make_type_converter {
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

macro_rules! make_value_extractor {
    ( $func:ident, $ty:ty, $val_ty:ty, $op:ident) => {
        fn $func(&self) -> Option<$val_ty> {
            if let Some(obj) = self.as_any().downcast_ref::<$ty>() {
                Some(obj.value().$op())
            } else {
                None
            }
        }
    };
}

macro_rules! make_unary_op {
    ( $meth:ident, $op:literal, $result:ty ) => {
        fn $meth(&self, _ctx: &RuntimeContext) -> $result {
            Err(RuntimeErr::new_type_err(format!(
                "Unary operator {} ({}) not implemented for {}",
                $op,
                stringify!($meth),
                self.class()
            )))
        }
    };
}

macro_rules! make_bin_op {
    ( $func:ident, $op:literal, $result:ty ) => {
        fn $func(&self, _rhs: &dyn Object, _ctx: &RuntimeContext) -> $result {
            Err(RuntimeErr::new_type_err(format!(
                "Binary operator {} ({}) not implemented for {}",
                $op,
                stringify!($func),
                self.class()
            )))
        }
    };
}

/// Represents an instance of some type (AKA "class").
pub trait Object {
    fn class(&self) -> &TypeRef;
    fn as_any(&self) -> &dyn Any;

    fn id(&self) -> usize {
        let p = self as *const Self;
        let p = p as *const () as usize;
        p
    }

    fn type_name(&self) -> String {
        self.class().name()
    }

    fn qualified_type_name(&self) -> String {
        self.class().qualified_name()
    }

    // Type checkers ---------------------------------------------------

    make_type_checker!(is_nil, Nil);
    make_type_checker!(is_bool, Bool);
    make_type_checker!(is_int, Int);
    make_type_checker!(is_float, Float);
    make_type_checker!(is_str, Str);
    make_type_checker!(is_tuple, Tuple);
    make_type_checker!(is_func, Func);
    make_type_checker!(is_builtin_func, BuiltinFunc);

    // Type converters -------------------------------------------------
    //
    // These convert objects to their concrete types.

    make_type_converter!(as_type, Type);
    make_type_converter!(as_func, Func);
    make_type_converter!(as_builtin_func, BuiltinFunc);
    make_type_converter!(as_tuple, Tuple);

    // Value extractors ------------------------------------------------
    //
    // These extract the inner value from an object.

    make_value_extractor!(int_val, Int, BigInt, clone);
    make_value_extractor!(str_val, Str, String, to_owned);

    // Unary operations ------------------------------------------------

    make_unary_op!(negate, "-", RuntimeObjResult);
    make_unary_op!(as_bool, "!!", RuntimeBoolResult);

    fn not(&self, ctx: &RuntimeContext) -> RuntimeBoolResult {
        match self.as_bool(ctx) {
            Ok(true) => Ok(false),
            Ok(false) => Ok(true),
            err => err,
        }
    }

    // Binary operations -----------------------------------------------

    fn is_equal(&self, rhs: &dyn Object, _ctx: &RuntimeContext) -> bool {
        // This duplicates ObjectExt::is(), but that method can't be
        // used here.
        self.id() == rhs.id()
    }

    fn not_equal(&self, rhs: &dyn Object, ctx: &RuntimeContext) -> bool {
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

    // Attributes (accessed by name) -----------------------------------

    /// Attributes that all objects have. This should NOT be overridden.
    fn get_base_attr(&self, name: &str, ctx: &RuntimeContext) -> Option<ObjectRef> {
        let attr = match name {
            "$id" => ctx.builtins.new_int(self.id()),
            "$type_name" => ctx.builtins.new_str(self.type_name()),
            "$qualified_type_name" => ctx.builtins.new_str(self.qualified_type_name()),
            _ => return None,
        };
        Some(attr)
    }

    fn get_attr(&self, name: &str, ctx: &RuntimeContext) -> GetAttrResult {
        if let Some(attr) = self.get_base_attr(name, ctx) {
            Ok(attr)
        } else {
            Err(self.attr_does_not_exist(name))
        }
    }

    fn set_attr(
        &self,
        name: &str,
        _value: ObjectRef,
        _ctx: &RuntimeContext,
    ) -> SetAttrResult {
        Err(RuntimeErr::new_attr_cannot_be_set(
            self.qualified_type_name().as_str(),
            name,
        ))
    }

    fn attr_does_not_exist(&self, name: &str) -> RuntimeErr {
        RuntimeErr::new_attr_does_not_exist(self.qualified_type_name().as_str(), name)
    }

    // Items (accessed by index) ---------------------------------------

    fn get_item(&self, index: &BigInt, _ctx: &RuntimeContext) -> GetAttrResult {
        Err(self.item_does_not_exist(index))
    }

    fn set_item(
        &self,
        index: &BigInt,
        _value: ObjectRef,
        _ctx: &RuntimeContext,
    ) -> SetAttrResult {
        Err(RuntimeErr::new_item_cannot_be_set(
            self.qualified_type_name(),
            index.clone(),
        ))
    }

    fn item_does_not_exist(&self, index: &BigInt) -> RuntimeErr {
        RuntimeErr::new_item_does_not_exist(
            self.qualified_type_name().as_str(),
            index.clone(),
        )
    }
}

// Object extensions ---------------------------------------------------

/// Methods that aren't "object safe"
pub trait ObjectExt: Object {
    fn is(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<T: Object + ?Sized> ObjectExt for T {}

// Display -------------------------------------------------------------

/// Downcast Object to concrete type/object and display that.
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

impl fmt::Display for dyn Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_instance!(
            f,
            self,
            Bool,
            BuiltinFunc,
            Type,
            Custom,
            Float,
            Func,
            Int,
            Namespace,
            Nil,
            Str,
            Tuple
        );
        // Fallback
        let class = self.class();
        let id = self.id();
        write!(f, "{class} object @ {id}")
    }
}

impl fmt::Debug for dyn Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_instance!(
            f,
            self,
            Bool,
            BuiltinFunc,
            Type,
            Custom,
            Float,
            Func,
            Int,
            Namespace,
            Nil,
            Str,
            Tuple
        );
        // Fallback
        write!(f, "{self}")
    }
}
