use std::any::Any;
use std::fmt;
use std::rc::Rc;

use crate::vm::{
    RuntimeBoolResult, RuntimeContext, RuntimeErr, RuntimeErrKind, RuntimeResult,
};

use super::class::TypeRef;

pub type ObjectRef = Rc<dyn Object>;

macro_rules! make_unary_op {
    ( $meth:ident, $op:literal, $result:ty ) => {
        fn $meth(&self, _ctx: &RuntimeContext) -> $result {
            Err(RuntimeErr::new_type_error(format!(
                "Unary operator {} ({}) not implemented for type {}",
                $op,
                stringify!($meth),
                self.class().name()
            )))
        }
    };
}

macro_rules! make_bin_op {
    ( $meth:ident, $op:literal, $result:ty ) => {
        fn $meth(&self, _rhs: &ObjectRef, _ctx: &RuntimeContext) -> $result {
            Err(RuntimeErr::new_type_error(format!(
                "Binary operator {} ({}) not implemented for type {}",
                $op,
                stringify!($meth),
                self.class().name()
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

    fn name(&self) -> String {
        self.class().name().to_owned()
    }

    // Unary operations ------------------------------------------------

    make_unary_op!(negate, "-", RuntimeResult);
    make_unary_op!(as_bool, "!!", RuntimeBoolResult);

    fn not(&self, ctx: &RuntimeContext) -> RuntimeBoolResult {
        match self.as_bool(ctx) {
            Ok(true) => Ok(false),
            Ok(false) => Ok(true),
            err => err,
        }
    }

    // Binary operations -----------------------------------------------

    make_bin_op!(is_equal, "==", RuntimeBoolResult);
    fn not_equal(&self, rhs: &ObjectRef, ctx: &RuntimeContext) -> RuntimeBoolResult {
        self.is_equal(rhs, ctx).map(|equal| !equal)
    }

    make_bin_op!(less_than, "<", RuntimeBoolResult);
    make_bin_op!(greater_than, ">", RuntimeBoolResult);

    make_bin_op!(pow, "^", RuntimeResult);
    make_bin_op!(modulo, "%", RuntimeResult);
    make_bin_op!(mul, "*", RuntimeResult);
    make_bin_op!(div, "/", RuntimeResult);
    make_bin_op!(floor_div, "//", RuntimeResult);
    make_bin_op!(add, "+", RuntimeResult);
    make_bin_op!(sub, "-", RuntimeResult);
    make_bin_op!(and, "&&", RuntimeBoolResult);
    make_bin_op!(or, "||", RuntimeBoolResult);

    // Attributes ------------------------------------------------------

    fn get_attribute(&self, name: &str) -> Result<ObjectRef, RuntimeErr> {
        Err(RuntimeErr::new(RuntimeErrKind::AttributeDoesNotExist(name.to_owned())))
    }

    fn set_attribute(&self, name: &str, _value: ObjectRef) -> Result<(), RuntimeErr> {
        Err(RuntimeErr::new(RuntimeErrKind::AttributeCannotBeSet(name.to_owned())))
    }
}

// Object extensions ---------------------------------------------------

/// Methods that aren't "object safe"
pub trait ObjectExt: Object {
    fn is(&self, other: &Self) -> bool {
        self.class().is(&other.class()) && self.id() == other.id()
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
            super::nil::Nil,
            super::bool::Bool,
            super::int::Int,
            super::float::Float,
            super::str::Str,
            super::tuple::Tuple,
            super::func::Func,
            super::complex::ComplexObject
        );
        // Fallback
        write!(f, "{}()", self.class())
    }
}

impl fmt::Debug for dyn Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_instance!(
            f,
            self,
            super::nil::Nil,
            super::bool::Bool,
            super::int::Int,
            super::float::Float,
            super::str::Str,
            super::tuple::Tuple,
            super::func::Func,
            super::complex::ComplexObject
        );
        // Fallback
        write!(f, "{} object @ {:?} -> {}", self.class(), self.id(), self)
    }
}
