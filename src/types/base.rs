//! Type System
use std::any::{Any, TypeId};
use std::fmt;
use std::sync::{Arc, RwLock};

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::modules;
use crate::vm::{RuntimeBoolResult, RuntimeErr, RuntimeObjResult, RuntimeResult, VM};

use super::create;
use super::result::{Args, GetAttrResult, SetAttrResult};
use super::util::args_to_str;

use super::bool::{Bool, BoolType};
use super::bound_func::{BoundFunc, BoundFuncType};
use super::builtin_func::{BuiltinFunc, BuiltinFuncType};
use super::class::{Type, TypeType};
use super::closure::{Closure, ClosureType};
use super::custom::{CustomObj, CustomType};
use super::float::{Float, FloatType};
use super::func::{Func, FuncType};
use super::int::{Int, IntType};
use super::list::{List, ListType};
use super::map::{Map, MapType};
use super::module::{Module, ModuleType};
use super::nil::{Nil, NilType};
use super::ns::Namespace;
use super::str::{Str, StrType};
use super::tuple::{Tuple, TupleType};

pub type TypeRef = Arc<RwLock<dyn TypeTrait>>;
pub type ObjectRef = Arc<RwLock<dyn ObjectTrait>>;

// Type Trait ----------------------------------------------------------

/// Types in the system are backed by an implementation of `TypeTrait`.
/// Each type implementation will be instantiated exactly once (i.e.,
/// types are singletons). Example: `IntType`.
pub trait TypeTrait {
    fn name(&self) -> &str;
    fn full_name(&self) -> &str;
    fn namespace(&self) -> &Namespace;

    fn module(&self) -> ObjectRef {
        modules::BUILTINS.clone()
    }

    fn id(&self) -> usize {
        let p = self as *const Self;
        p as *const () as usize
    }

    fn is(&self, other: &dyn TypeTrait) -> bool {
        self.id() == other.id()
    }
}

// Object Trait --------------------------------------------------------

/// Create associated function to check is object ref a specific impl
/// type.
macro_rules! make_type_checker {
    ( $func:ident, $ty:ty) => {
        fn $func(&self) -> bool {
            self.as_any().type_id() == TypeId::of::<$ty>()
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
    ( $func:ident, $ty:ty, $val_ty:ty ) => {
        fn $func(&self) -> Option<$val_ty> {
            self.as_any().downcast_ref::<$ty>().map(|obj| obj.value())
        }
    };
}

/// Create associated unary op function.
macro_rules! make_unary_op {
    ( $meth:ident, $op:literal, $result:ty ) => {
        fn $meth(&self) -> $result {
            Err(RuntimeErr::new_type_err(format!(
                "Unary operator {} ({}) not implemented for {}",
                $op,
                stringify!($meth),
                self.type_obj().read().unwrap()
            )))
        }
    };
}

/// Create associated binary op function.
macro_rules! make_bin_op {
    ( $func:ident, $op:literal, $result:ty ) => {
        fn $func(&self, _rhs: &dyn ObjectTrait) -> $result {
            Err(RuntimeErr::new_type_err(format!(
                "Binary operator {} ({}) not implemented for {}",
                $op,
                stringify!($func),
                self.type_obj().read().unwrap()
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
    fn namespace(&self) -> &Namespace;

    fn id(&self) -> usize {
        let p = self as *const Self;
        p as *const () as usize
    }

    fn id_obj(&self) -> ObjectRef {
        create::new_int(self.id())
    }

    fn module(&self) -> ObjectRef {
        let class = self.class();
        let class = class.read().unwrap();
        class.module().clone()
    }

    // Attributes (accessed by name) -----------------------------------

    fn get_attr(&self, name: &str) -> GetAttrResult {
        if name == "$type" {
            return Ok(self.type_obj().clone());
        }
        if name == "$module" {
            let module = self.class().read().unwrap().module().clone();
            return Ok(module);
        }
        if name == "$id" {
            return Ok(self.id_obj());
        }
        if let Some(obj) = self.namespace().get_obj(name) {
            return Ok(obj);
        }
        if let Some(obj) = self.type_obj().read().unwrap().namespace().get_obj(name) {
            return Ok(obj);
        }
        Err(self.attr_does_not_exist(name))
    }

    fn set_attr(&mut self, name: &str, _value: ObjectRef) -> SetAttrResult {
        Err(RuntimeErr::new_attr_cannot_be_set(
            self.class().read().unwrap().full_name(),
            name,
        ))
    }

    fn attr_does_not_exist(&self, name: &str) -> RuntimeErr {
        RuntimeErr::new_attr_does_not_exist(
            self.class().read().unwrap().full_name(),
            name,
        )
    }

    // Items (accessed by index) ---------------------------------------

    fn get_item(&self, index: usize) -> GetAttrResult {
        Err(self.item_does_not_exist(index))
    }

    fn set_item(&mut self, index: usize, _value: ObjectRef) -> GetAttrResult {
        Err(RuntimeErr::new_item_cannot_be_set(
            self.class().read().unwrap().full_name(),
            index,
        ))
    }

    fn item_does_not_exist(&self, index: usize) -> RuntimeErr {
        RuntimeErr::new_item_does_not_exist(
            self.class().read().unwrap().full_name(),
            index,
        )
    }

    fn index_out_of_bounds(&self, index: usize) -> RuntimeErr {
        RuntimeErr::new_index_out_of_bounds(
            self.class().read().unwrap().full_name(),
            index,
        )
    }

    // Type checkers ---------------------------------------------------

    make_type_checker!(is_type_type, TypeType);
    make_type_checker!(is_bool_type, BoolType);
    make_type_checker!(is_bound_func_type, BoundFuncType);
    make_type_checker!(is_builtin_func_type, BuiltinFuncType);
    make_type_checker!(is_closure_type, ClosureType);
    make_type_checker!(is_float_type, FloatType);
    make_type_checker!(is_func_type, FuncType);
    make_type_checker!(is_int_type, IntType);
    make_type_checker!(is_list_type, ListType);
    make_type_checker!(is_map_type, MapType);
    make_type_checker!(is_mod_type, ModuleType);
    make_type_checker!(is_nil_type, NilType);
    make_type_checker!(is_str_type, StrType);
    make_type_checker!(is_tuple_type, TupleType);

    make_type_checker!(is_type, Type);
    make_type_checker!(is_bool, Bool);
    make_type_checker!(is_bound_func, BoundFunc);
    make_type_checker!(is_builtin_func, BuiltinFunc);
    make_type_checker!(is_closure, Closure);
    make_type_checker!(is_float, Float);
    make_type_checker!(is_func, Func);
    make_type_checker!(is_int, Int);
    make_type_checker!(is_list, List);
    make_type_checker!(is_map, Map);
    make_type_checker!(is_mod, Module);
    make_type_checker!(is_nil, Nil);
    make_type_checker!(is_str, Str);
    make_type_checker!(is_tuple, Tuple);

    // Downcasters -----------------------------------------------------
    //
    // These downcast object refs to their concrete types.

    make_down_to!(down_to_type_type, TypeType);
    make_down_to!(down_to_bool_type, BoolType);
    make_down_to!(down_to_bound_func_type, BoundFuncType);
    make_down_to!(down_to_builtin_func_type, BuiltinFuncType);
    make_down_to!(down_to_closure_type, ClosureType);
    make_down_to!(down_to_float_type, FloatType);
    make_down_to!(down_to_func_type, FuncType);
    make_down_to!(down_to_list_type, ListType);
    make_down_to!(down_to_int_type, IntType);
    make_down_to!(down_to_map_type, MapType);
    make_down_to!(down_to_mod_type, ModuleType);
    make_down_to!(down_to_nil_type, NilType);
    make_down_to!(down_to_str_type, StrType);
    make_down_to!(down_to_tuple_type, TupleType);

    make_down_to!(down_to_type, Type);
    make_down_to!(down_to_bool, Bool);
    make_down_to!(down_to_bound_func, BoundFunc);
    make_down_to!(down_to_builtin_func, BuiltinFunc);
    make_down_to!(down_to_closure, Closure);
    make_down_to!(down_to_float, Float);
    make_down_to!(down_to_func, Func);
    make_down_to!(down_to_int, Int);
    make_down_to!(down_to_list, List);
    make_down_to!(down_to_map, Map);
    make_down_to!(down_to_mod, Module);
    make_down_to!(down_to_nil, Nil);
    make_down_to!(down_to_str, Str);
    make_down_to!(down_to_tuple, Tuple);

    // Value extractors ------------------------------------------------
    //
    // These extract the inner value from an object.

    make_value_extractor!(get_bool_val, Bool, &bool);
    make_value_extractor!(get_float_val, Float, &f64);
    make_value_extractor!(get_int_val, Int, &BigInt);
    make_value_extractor!(get_str_val, Str, &str);

    fn get_usize_val(&self) -> Option<usize> {
        if let Some(int) = self.get_int_val() {
            int.to_usize()
        } else {
            None
        }
    }

    // Unary operations ------------------------------------------------

    make_unary_op!(negate, "-", RuntimeObjResult);
    make_unary_op!(bool_val, "!!", RuntimeBoolResult);

    fn not(&self) -> RuntimeBoolResult {
        match self.bool_val() {
            Ok(true) => Ok(false),
            Ok(false) => Ok(true),
            err => err,
        }
    }

    // Binary operations -----------------------------------------------

    fn is(&self, other: &dyn ObjectTrait) -> bool {
        self.id() == other.id()
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        self.is(rhs)
    }

    fn not_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        !self.is_equal(rhs)
    }

    make_bin_op!(and, "&&", RuntimeBoolResult);
    make_bin_op!(or, "||", RuntimeBoolResult);
    make_bin_op!(less_than, "<", RuntimeBoolResult);
    make_bin_op!(greater_than, ">", RuntimeBoolResult);

    make_bin_op!(pow, "^", RuntimeObjResult);
    make_bin_op!(modulo, "%", RuntimeObjResult);
    make_bin_op!(mul, "*", RuntimeObjResult);
    make_bin_op!(div, "/", RuntimeObjResult);
    make_bin_op!(floor_div, "//", RuntimeObjResult);
    make_bin_op!(add, "+", RuntimeObjResult);
    make_bin_op!(sub, "-", RuntimeObjResult);

    // Call ------------------------------------------------------------

    // This is here so that functions can be called directly, in
    // particular so that user functions can be called from builtin
    // functions.
    fn call(&self, args: Args, _vm: &mut VM) -> RuntimeResult {
        log::trace!("BEGIN: base call");
        log::trace!("ARGS: {}", args_to_str(&args));
        Err(self.not_callable())
    }

    fn not_callable(&self) -> RuntimeErr {
        RuntimeErr::new_not_callable(self.class().read().unwrap().full_name())
    }
}

// Display -------------------------------------------------------------

macro_rules! write_type_instance {
    ( $f:ident, $t:ident, $($A:ty),+ ) => { $(
        if let Some(t) = $t.as_any().downcast_ref::<$A>() {
            return write!($f, "<{}>", t.full_name());
        }
    )+ };
}

macro_rules! debug_type_instance {
    ( $f:ident, $t:ident, $($A:ty),+ ) => { $(
        if let Some(t) = $t.as_any().downcast_ref::<$A>() {
            return write!($f, "<{}> @ {}", t.full_name(), ObjectTrait::id(t));
        }
    )+ };
}

macro_rules! write_instance {
    ( $f:ident, $i:ident, $($A:ty),+ ) => { $(
        if let Some(i) = $i.as_any().downcast_ref::<$A>() {
            return write!($f, "{i}");
        }
    )+ };
}

macro_rules! debug_instance {
    ( $f:ident, $i:ident, $($A:ty),+ ) => { $(
        if let Some(i) = $i.as_any().downcast_ref::<$A>() {
            return write!($f, "{i:?}");
        }
    )+ };
}

impl fmt::Display for dyn TypeTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.full_name())
    }
}

impl fmt::Debug for dyn TypeTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{} @ {}>", self.full_name(), self.id())
    }
}

impl fmt::Display for dyn ObjectTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_type_instance!(
            f,
            self,
            TypeType,
            BoolType,
            BoundFuncType,
            BuiltinFuncType,
            ClosureType,
            CustomType,
            FloatType,
            FuncType,
            IntType,
            ListType,
            MapType,
            ModuleType,
            NilType,
            StrType,
            TupleType
        );
        write_instance!(
            f,
            self,
            Type,
            Bool,
            BoundFunc,
            BuiltinFunc,
            Closure,
            CustomObj,
            Float,
            Func,
            Int,
            List,
            Map,
            Module,
            Nil,
            Str,
            Tuple
        );
        panic!("Display must be defined");
    }
}

impl fmt::Debug for dyn ObjectTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_type_instance!(
            f,
            self,
            TypeType,
            BoolType,
            BoundFuncType,
            BuiltinFuncType,
            ClosureType,
            CustomType,
            FloatType,
            FuncType,
            IntType,
            ListType,
            MapType,
            ModuleType,
            NilType,
            StrType,
            TupleType
        );
        debug_instance!(
            f,
            self,
            Type,
            Bool,
            BoundFunc,
            BuiltinFunc,
            Closure,
            CustomObj,
            Float,
            Func,
            Int,
            List,
            Map,
            Module,
            Nil,
            Str,
            Tuple
        );
        panic!("Debug must be defined");
    }
}
