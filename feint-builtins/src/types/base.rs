//! Type System
use std::any::{Any, TypeId};
use std::fmt;
use std::sync::{Arc, RwLock};

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use feint_code_gen::*;

use crate::BUILTINS;

use super::func_trait::FuncTrait;
use super::ns::Namespace;

use super::always::Always;
use super::bool::Bool;
use super::bound_func::BoundFunc;
use super::cell::Cell;
use super::class::Type;
use super::closure::Closure;
use super::custom::CustomObj;
use super::err::ErrObj;
use super::err_type::ErrTypeObj;
use super::file::File;
use super::float::Float;
use super::func::Func;
use super::int::Int;
use super::intrinsic_func::IntrinsicFunc;
use super::iterator::FIIterator;
use super::list::List;
use super::map::Map;
use super::module::Module;
use super::nil::Nil;
use super::prop::Prop;
use super::str::Str;
use super::tuple::Tuple;

pub type TypeRef = obj_ref_t!(Type);
pub type ObjectRef = obj_ref_t!(dyn ObjectTrait);

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
    ( $meth:ident, $op:literal, $ty:ty ) => {
        fn $meth(&self) -> Option<$ty> {
            None
        }
    };
}

/// Create associated binary op function.
macro_rules! make_bin_op {
    ( $func:ident, $op:literal, $ty:ty ) => {
        fn $func(&self, _rhs: &dyn ObjectTrait) -> Option<$ty> {
            None
        }
    };
}

/// Objects in the system--instances of types--are backed by an
/// implementation of `ObjectTrait`. Example: `Int`.
pub trait ObjectTrait {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn class(&self) -> TypeRef;

    /// Each object has a namespace that holds its attributes.
    fn ns(&self) -> &Namespace;
    fn ns_mut(&mut self) -> &mut Namespace;

    fn id(&self) -> usize {
        let p = self as *const Self;
        p as *const () as usize
    }

    fn id_obj(&self) -> ObjectRef {
        // TODO: Cache?
        BUILTINS.int(self.id())
    }

    /// XXX: This resolves to `std` unless overridden.
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
        self.base_get_attr(name, this)
    }

    fn base_get_attr(&self, name: &str, this: ObjectRef) -> ObjectRef {
        // Special attributes that *cannot* be overridden --------------
        if name == "$id" {
            return self.id_obj();
        }

        // if name == "$module" {
        //     return self.module();
        // }

        if name == "$type" {
            return self.class();
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
            let items = names.iter().map(|n| BUILTINS.str(n)).collect();
            return BUILTINS.tuple(items);
        }

        // TODO: Convert to builtin function
        // if name == "$dis" {
        //     // User functions, bound functions wrapping user functions,
        //     // and closures wrapping user functions can be disassembled.
        //     if let Some(f) = self.down_to_func() {
        //         let mut dis = Disassembler::new();
        //         dis.disassemble(f.code());
        //     } else if let Some(b) = self.down_to_bound_func() {
        //         let f = b.func();
        //         let f = f.read().unwrap();
        //         if let Some(f) = f.down_to_func() {
        //             let mut dis = Disassembler::new();
        //             dis.disassemble(f.code());
        //         } else {
        //             eprintln!("Cannot disassemble bound func: {}", b);
        //         }
        //     } else if let Some(c) = self.down_to_closure() {
        //         let f = c.func();
        //         let f = f.read().unwrap();
        //         if let Some(f) = f.down_to_func() {
        //             let mut dis = Disassembler::new();
        //             dis.disassemble(f.code());
        //         } else {
        //             eprintln!("Cannot disassemble closure: {}", c);
        //         }
        //     } else if let Some(m) = self.down_to_mod() {
        //         let mut dis = Disassembler::new();
        //         dis.disassemble(m.code());
        //     } else {
        //         eprintln!("Cannot disassemble object: {}", &*this.read().unwrap());
        //     }
        //     return BUILTINS.nil();
        // }

        // Instance attributes -----------------------------------------
        //
        // Check instance then instance type.

        if let Some(obj) = self.ns().get(name) {
            return obj;
        }

        if let Some(obj) = self.class().read().unwrap().ns().get(name) {
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
                BUILTINS.bool(!err.retrieve_bool_val())
            } else {
                BUILTINS.bool(true)
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
                BUILTINS.err_with_responds_to_bool(
                    err.kind.clone(),
                    err.message.as_str(),
                    this.clone(),
                )
            } else {
                BUILTINS.ok_err()
            };
        }

        if name == "to_str" {
            return if self.is_str() {
                this.clone()
            } else {
                BUILTINS.str(this.read().unwrap().to_string())
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
        BUILTINS.attr_not_found_err(name, obj)
    }

    // Items (accessed by index) ---------------------------------------

    fn get_item(&self, index: usize, this: ObjectRef) -> ObjectRef {
        // TODO: The default should be a "does not support" indexing err
        BUILTINS.index_out_of_bounds_err(index, this)
    }

    fn set_item(
        &mut self,
        index: usize,
        this: ObjectRef,
        _value: ObjectRef,
    ) -> ObjectRef {
        // TODO: The default should be a "does not support" indexing err
        BUILTINS.index_out_of_bounds_err(index, this)
    }

    fn index_out_of_bounds(&self, index: usize, this: ObjectRef) -> ObjectRef {
        BUILTINS.index_out_of_bounds_err(index, this)
    }

    // Type checkers ---------------------------------------------------

    make_type_checker!(is_type, Type);
    make_type_checker!(is_always, Always);
    make_type_checker!(is_bool, Bool);
    make_type_checker!(is_bound_func, BoundFunc);
    make_type_checker!(is_intrinsic_func, IntrinsicFunc);
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

    fn is_immutable(&self) -> bool {
        !(self.is_cell() || self.is_file() || self.is_list() || self.is_map())
    }

    fn is_seq(&self) -> bool {
        self.is_list() || self.is_tuple()
    }

    // Downcasters -----------------------------------------------------
    //
    // These downcast object refs to their concrete types.

    make_down_to!(down_to_type, Type);
    make_down_to!(down_to_always, Always);
    make_down_to!(down_to_bool, Bool);
    make_down_to!(down_to_bound_func, BoundFunc);
    make_down_to!(down_to_intrinsic_func, IntrinsicFunc);
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
        let f: &dyn FuncTrait = if let Some(f) = self.down_to_intrinsic_func() {
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

    make_unary_op!(negate, "-", ObjectRef);
    make_unary_op!(bool_val, "!!", bool);

    fn not(&self) -> Option<bool> {
        match self.bool_val() {
            Some(true) => Some(false),
            Some(false) => Some(true),
            None => None,
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
        let t = self.class();
        let t = t.read().unwrap();
        let u = rhs.class();
        let u = u.read().unwrap();
        t.is(&*u) && self.is_equal(rhs)
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        self.is(rhs) || rhs.is_always()
    }

    make_bin_op!(and, "&&", bool);
    make_bin_op!(or, "||", bool);
    make_bin_op!(less_than, "<", bool);
    make_bin_op!(greater_than, ">", bool);

    make_bin_op!(pow, "^", ObjectRef);
    make_bin_op!(modulo, "%", ObjectRef);
    make_bin_op!(mul, "*", ObjectRef);
    make_bin_op!(div, "/", ObjectRef);
    make_bin_op!(floor_div, "//", ObjectRef);
    make_bin_op!(add, "+", ObjectRef);
    make_bin_op!(sub, "-", ObjectRef);
}

// Display -------------------------------------------------------------

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

impl fmt::Display for dyn ObjectTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_instance!(
            f,
            self,
            Always,
            Bool,
            BoundFunc,
            IntrinsicFunc,
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
        debug_instance!(
            f,
            self,
            Type,
            Always,
            Bool,
            BoundFunc,
            IntrinsicFunc,
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
