//! Type System
use std::any::{Any, TypeId};
use std::fmt;
use std::sync::{Arc, RwLock};

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::dis::Disassembler;
use crate::modules::std::BUILTINS;
use crate::types::FuncTrait;
use crate::vm::{RuntimeBoolResult, RuntimeErr, RuntimeObjResult};

use super::gen;
use super::new;
use super::ns::Namespace;

use super::always::{Always, AlwaysType};
use super::bool::{Bool, BoolType};
use super::bound_func::{BoundFunc, BoundFuncType};
use super::builtin_func::{BuiltinFunc, BuiltinFuncType};
use super::cell::{Cell, CellType};
use super::class::{Type, TypeType};
use super::closure::{Closure, ClosureType};
use super::custom::{CustomObj, CustomType};
use super::err::{ErrObj, ErrType};
use super::err_type::{ErrTypeObj, ErrTypeType};
use super::file::{File, FileType};
use super::float::{Float, FloatType};
use super::func::{Func, FuncType};
use super::int::{Int, IntType};
use super::iterator::{FIIterator, IteratorType};
use super::list::{List, ListType};
use super::map::{Map, MapType};
use super::module::{Module, ModuleType};
use super::nil::{Nil, NilType};
use super::prop::{Prop, PropType};
use super::str::{Str, StrType};
use super::tuple::{Tuple, TupleType};

pub type TypeRef = gen::obj_ref_t!(dyn TypeTrait);
pub type ObjectRef = gen::obj_ref_t!(dyn ObjectTrait);

// Type Trait ----------------------------------------------------------

/// Types in the system are backed by an implementation of `TypeTrait`.
/// Each type implementation will be instantiated exactly once (i.e.,
/// types are singletons). Example: `IntType`.
pub trait TypeTrait {
    fn name(&self) -> &str;
    fn full_name(&self) -> &str;
    fn ns(&self) -> &Namespace;

    fn module(&self) -> ObjectRef {
        BUILTINS.clone()
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

/// Create associated function to check is object ref is a specific impl
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

/// Create associated function to downcast from object ref to mut impl.
macro_rules! make_down_to_mut {
    ( $func:ident, $ty:ty) => {
        fn $func(&mut self) -> Option<&mut $ty> {
            self.as_any_mut().downcast_mut::<$ty>()
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
            Err(RuntimeErr::type_err(format!(
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
            Err(RuntimeErr::type_err(format!(
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
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Get an instance's type as a type. This is needed to retrieve
    /// type level attributes.
    fn class(&self) -> TypeRef;

    /// Get an instance's type as an object. This is needed so the type
    /// can be used in object contexts.
    fn type_obj(&self) -> ObjectRef;

    /// Each object has a namespace that holds its attributes.
    fn ns(&self) -> &Namespace;
    fn ns_mut(&mut self) -> &mut Namespace;

    /// Cast object to type, if possible.
    fn as_type(&self) -> Option<&dyn TypeTrait>;

    fn id(&self) -> usize {
        let p = self as *const Self;
        p as *const () as usize
    }

    fn id_obj(&self) -> ObjectRef {
        // TODO: Cache?
        new::int(self.id())
    }

    /// XXX: This resolves to `std.builtins` unless overridden.
    fn module(&self) -> ObjectRef {
        let class = self.class();
        let class = class.read().unwrap();
        class.module().clone()
    }

    // Attributes (accessed by name) -----------------------------------

    /// Get attribute.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the attribute.
    /// * `this` - The object reference wrapping `self`. The main use
    ///    for this currently is to allow the object reference to be
    ///    returned as the result without having to clone the inner
    ///    object.
    ///    TODO: There's probably a more elegant way to do this, but it
    ///          might require a bit of re-architecting.
    fn get_attr(&self, name: &str, this: ObjectRef) -> ObjectRef {
        // Special attributes that *cannot* be overridden --------------
        if name == "$id" {
            return self.id_obj();
        }

        if name == "$module" {
            return self.module();
        }

        if name == "$type" {
            return self.type_obj();
        }

        if name == "$names" {
            let class = self.class();
            let class = class.read().unwrap();
            let class_ns = class.ns();
            let obj_ns = self.ns();
            let mut names: Vec<String> =
                class_ns.iter().map(|(n, _)| n).cloned().collect();
            names.extend(obj_ns.iter().map(|(n, _)| n).cloned());
            names.sort();
            names.dedup();
            let items = names.iter().map(new::str).collect();
            return new::tuple(items);
        }

        if name == "$dis" {
            // User functions, bound functions wrapping user functions,
            // and closures wrapping user functions can be disassembled.
            if let Some(f) = self.down_to_func() {
                let mut dis = Disassembler::new();
                dis.disassemble(f.code());
            } else if let Some(b) = self.down_to_bound_func() {
                let f = b.func();
                let f = f.read().unwrap();
                if let Some(f) = f.down_to_func() {
                    let mut dis = Disassembler::new();
                    dis.disassemble(f.code());
                } else {
                    eprintln!("Cannot disassemble bound func: {}", b);
                }
            } else if let Some(c) = self.down_to_closure() {
                let f = c.func();
                let f = f.read().unwrap();
                if let Some(f) = f.down_to_func() {
                    let mut dis = Disassembler::new();
                    dis.disassemble(f.code());
                } else {
                    eprintln!("Cannot disassemble closure: {}", c);
                }
            } else if let Some(m) = self.down_to_mod() {
                let mut dis = Disassembler::new();
                dis.disassemble(m.code());
            } else {
                eprintln!("Cannot disassemble object: {}", &*this.read().unwrap());
            }
            return new::nil();
        }

        // Instance attributes -----------------------------------------
        //
        // Check instance then instance type.

        if let Some(obj) = self.ns().get_obj(name) {
            return obj;
        }

        if let Some(obj) = self.type_obj().read().unwrap().ns().get_obj(name) {
            return obj;
        }

        // Public attributes that *can* be overridden ------------------

        // OK status associated with this object.
        //
        // If this object *is* an error, `false` is returned unless the
        // object is the special ok error type, in which case `true` is
        // returned.
        //
        // If this object *is not* an error, `true` is returned.
        //
        // NOT: This needs to be after instance attribute lookup so that
        //      `ErrType.ok` returns the OK err type rather than a bool.
        if name == "ok" {
            let this = this.read().unwrap();
            return if let Some(err) = this.down_to_err() {
                new::bool(!err.retrieve_bool_val())
            } else {
                new::bool(true)
            };
        }

        // Error object associated with this object.
        //
        // If this object *is* an error, a copy of the error that
        // responds to bool is returned.
        //
        // If this object *is not* an error, the singleton OK object
        // that responds to bool is returned.
        if name == "err" {
            return if let Some(err) = this.read().unwrap().down_to_err() {
                new::err_with_responds_to_bool(
                    err.kind.clone(),
                    err.message.as_str(),
                    this.clone(),
                )
            } else {
                new::ok_err()
            };
        }

        if name == "to_str" {
            return if self.is_str() {
                this.clone()
            } else {
                new::str(this.read().unwrap().to_string())
            };
        }

        self.attr_not_found(name, this)
    }

    /// Set attribute.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the attribute
    /// * `value` - The new value for the attribute
    /// * `this` - The object reference wrapping `self` (see note on
    ///   `get_attr()`.
    fn set_attr(
        &mut self,
        name: &str,
        _value: ObjectRef,
        this: ObjectRef,
    ) -> ObjectRef {
        // TODO: The default should be a "does not support" attr access
        self.attr_not_found(name, this)
    }

    fn attr_not_found(&self, name: &str, obj: ObjectRef) -> ObjectRef {
        new::attr_not_found_err(name, obj)
    }

    // Items (accessed by index) ---------------------------------------

    fn get_item(&self, index: usize, this: ObjectRef) -> ObjectRef {
        // TODO: The default should be a "does not support" indexing err
        new::index_out_of_bounds_err(index, this)
    }

    fn set_item(
        &mut self,
        index: usize,
        this: ObjectRef,
        _value: ObjectRef,
    ) -> ObjectRef {
        // TODO: The default should be a "does not support" indexing err
        new::index_out_of_bounds_err(index, this)
    }

    fn index_out_of_bounds(&self, index: usize, this: ObjectRef) -> ObjectRef {
        new::index_out_of_bounds_err(index, this)
    }

    // Type checkers ---------------------------------------------------

    make_type_checker!(is_type_type, TypeType);
    make_type_checker!(is_always_type, AlwaysType);
    make_type_checker!(is_bool_type, BoolType);
    make_type_checker!(is_bound_func_type, BoundFuncType);
    make_type_checker!(is_builtin_func_type, BuiltinFuncType);
    make_type_checker!(is_cell_type, CellType);
    make_type_checker!(is_closure_type, ClosureType);
    make_type_checker!(is_err_type, ErrType);
    make_type_checker!(is_err_type_type, ErrTypeType);
    make_type_checker!(is_file_type, FileType);
    make_type_checker!(is_float_type, FloatType);
    make_type_checker!(is_func_type, FuncType);
    make_type_checker!(is_int_type, IntType);
    make_type_checker!(is_iterator_type, IteratorType);
    make_type_checker!(is_list_type, ListType);
    make_type_checker!(is_map_type, MapType);
    make_type_checker!(is_mod_type, ModuleType);
    make_type_checker!(is_nil_type, NilType);
    make_type_checker!(is_prop_type, PropType);
    make_type_checker!(is_str_type, StrType);
    make_type_checker!(is_tuple_type, TupleType);

    make_type_checker!(is_type, Type);
    make_type_checker!(is_always, Always);
    make_type_checker!(is_bool, Bool);
    make_type_checker!(is_bound_func, BoundFunc);
    make_type_checker!(is_builtin_func, BuiltinFunc);
    make_type_checker!(is_cell, Cell);
    make_type_checker!(is_closure, Closure);
    make_type_checker!(is_err, ErrObj);
    make_type_checker!(is_err_type_obj, ErrTypeObj);
    make_type_checker!(is_file, File);
    make_type_checker!(is_float, Float);
    make_type_checker!(is_func, Func);
    make_type_checker!(is_int, Int);
    make_type_checker!(is_iterator, FIIterator);
    make_type_checker!(is_list, List);
    make_type_checker!(is_map, Map);
    make_type_checker!(is_mod, Module);
    make_type_checker!(is_nil, Nil);
    make_type_checker!(is_prop, Prop);
    make_type_checker!(is_str, Str);
    make_type_checker!(is_tuple, Tuple);

    /// Is this object a type object?
    fn is_type_object(&self) -> bool {
        self.type_obj().read().unwrap().is_type_type()
    }

    // XXX: Currently, everything but `Cell`s are immutable. This
    //      anticipates types that are likely to be made mutable in the
    //      future.
    fn is_immutable(&self) -> bool {
        !(self.is_cell() || self.is_file() || self.is_list() || self.is_map())
    }

    fn is_seq(&self) -> bool {
        self.is_list() || self.is_tuple()
    }

    // Downcasters -----------------------------------------------------
    //
    // These downcast object refs to their concrete types.

    make_down_to!(down_to_type_type, TypeType);
    make_down_to!(down_to_always_type, AlwaysType);
    make_down_to!(down_to_bool_type, BoolType);
    make_down_to!(down_to_bound_func_type, BoundFuncType);
    make_down_to!(down_to_builtin_func_type, BuiltinFuncType);
    make_down_to!(down_to_cell_type, CellType);
    make_down_to!(down_to_closure_type, ClosureType);
    make_down_to!(down_to_err_type, ErrType);
    make_down_to!(down_to_err_type_type, ErrTypeType);
    make_down_to!(down_to_file_type, FileType);
    make_down_to!(down_to_float_type, FloatType);
    make_down_to!(down_to_func_type, FuncType);
    make_down_to!(down_to_list_type, ListType);
    make_down_to!(down_to_int_type, IntType);
    make_down_to!(down_to_iterator_type, IteratorType);
    make_down_to!(down_to_map_type, MapType);
    make_down_to!(down_to_mod_type, ModuleType);
    make_down_to!(down_to_nil_type, NilType);
    make_down_to!(down_to_prop_type, PropType);
    make_down_to!(down_to_str_type, StrType);
    make_down_to!(down_to_tuple_type, TupleType);

    make_down_to!(down_to_type, Type);
    make_down_to!(down_to_always, Always);
    make_down_to!(down_to_bool, Bool);
    make_down_to!(down_to_bound_func, BoundFunc);
    make_down_to!(down_to_builtin_func, BuiltinFunc);
    make_down_to!(down_to_cell, Cell);
    make_down_to_mut!(down_to_cell_mut, Cell);
    make_down_to!(down_to_closure, Closure);
    make_down_to!(down_to_err, ErrObj);
    make_down_to!(down_to_err_type_obj, ErrTypeObj);
    make_down_to!(down_to_file, File);
    make_down_to_mut!(down_to_file_mut, File);
    make_down_to!(down_to_float, Float);
    make_down_to!(down_to_func, Func);
    make_down_to!(down_to_int, Int);
    make_down_to!(down_to_iterator, FIIterator);
    make_down_to_mut!(down_to_iterator_mut, FIIterator);
    make_down_to!(down_to_list, List);
    make_down_to!(down_to_map, Map);
    make_down_to!(down_to_mod, Module);
    make_down_to_mut!(down_to_mod_mut, Module);
    make_down_to!(down_to_nil, Nil);
    make_down_to!(down_to_prop, Prop);
    make_down_to!(down_to_str, Str);
    make_down_to!(down_to_tuple, Tuple);

    fn as_func(&self) -> Option<&dyn FuncTrait> {
        let f: &dyn FuncTrait = if let Some(f) = self.down_to_builtin_func() {
            f
        } else if let Some(f) = self.down_to_func() {
            f
        } else if let Some(f) = self.down_to_closure() {
            f
        } else if let Some(f) = self.down_to_bound_func() {
            f
        } else {
            return None;
        };
        Some(f)
    }

    // Value extractors ------------------------------------------------
    //
    // These extract the inner value from an object.

    make_value_extractor!(get_bool_val, Bool, &bool);
    make_value_extractor!(get_cell_val, Cell, ObjectRef);
    make_value_extractor!(get_float_val, Float, &f64);
    make_value_extractor!(get_int_val, Int, &BigInt);
    make_value_extractor!(get_str_val, Str, &str);

    fn get_map_val(&self) -> Option<&Map> {
        if let Some(map) = self.down_to_map() {
            Some(map)
        } else {
            None
        }
    }

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

    /// This requires both objects to have the same type along with
    /// being equal. This will return `false` when compared with `@`.
    fn is_type_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if self.is(rhs) {
            return true;
        }
        let t = self.type_obj();
        let t = t.read().unwrap();
        let u = rhs.type_obj();
        let u = u.read().unwrap();
        t.is(&*u) && self.is_equal(rhs)
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        self.is(rhs) || rhs.is_always()
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

    fn not_callable(&self) -> RuntimeErr {
        RuntimeErr::not_callable(self.class().read().unwrap().full_name())
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
            AlwaysType,
            BoolType,
            BoundFuncType,
            BuiltinFuncType,
            CellType,
            ClosureType,
            CustomType,
            ErrType,
            ErrTypeType,
            FileType,
            FloatType,
            FuncType,
            IntType,
            IteratorType,
            ListType,
            MapType,
            ModuleType,
            NilType,
            PropType,
            StrType,
            TupleType
        );
        write_instance!(
            f,
            self,
            Type,
            Always,
            Bool,
            BoundFunc,
            BuiltinFunc,
            Cell,
            Closure,
            CustomObj,
            ErrObj,
            ErrTypeObj,
            File,
            Float,
            Func,
            Int,
            FIIterator,
            List,
            Map,
            Module,
            Nil,
            Prop,
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
            AlwaysType,
            BoolType,
            BoundFuncType,
            BuiltinFuncType,
            CellType,
            ClosureType,
            CustomType,
            ErrType,
            ErrTypeType,
            FileType,
            FloatType,
            FuncType,
            IntType,
            IteratorType,
            ListType,
            MapType,
            ModuleType,
            NilType,
            PropType,
            StrType,
            TupleType
        );
        debug_instance!(
            f,
            self,
            Type,
            Always,
            Bool,
            BoundFunc,
            BuiltinFunc,
            Cell,
            Closure,
            CustomObj,
            ErrObj,
            ErrTypeObj,
            File,
            Float,
            Func,
            Int,
            FIIterator,
            List,
            Map,
            Module,
            Nil,
            Prop,
            Str,
            Tuple
        );
        panic!("Debug must be defined");
    }
}
