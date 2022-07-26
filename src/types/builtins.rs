use std::rc::Rc;

use num_bigint::BigInt;
use num_traits::Num;

use crate::vm::Chunk;

use super::native::NativeFn;
use super::object::ObjectRef;

pub struct Builtins {
    pub nil_obj: Rc<super::nil::Nil>,
    pub true_obj: Rc<super::bool::Bool>,
    pub false_obj: Rc<super::bool::Bool>,
    pub empty_tuple: Rc<super::tuple::Tuple>,
}

impl Builtins {
    pub fn new() -> Self {
        // Singletons
        let nil_obj = Rc::new(super::nil::Nil::new());
        let true_obj = Rc::new(super::bool::Bool::new(true));
        let false_obj = Rc::new(super::bool::Bool::new(false));
        let empty_tuple = Rc::new(super::tuple::Tuple::new(vec![]));
        Self { nil_obj, true_obj, false_obj, empty_tuple }
    }

    // Builtin type constructors

    /// Convert Rust native bool to Bool object
    pub fn bool_obj_from_bool(&self, value: bool) -> ObjectRef {
        if value {
            self.true_obj.clone()
        } else {
            self.false_obj.clone()
        }
    }

    pub fn new_float<F: Into<f64>>(&self, value: F) -> ObjectRef {
        let value = value.into();
        Rc::new(super::float::Float::new(value))
    }

    pub fn new_float_from_string<S: Into<String>>(&self, value: S) -> ObjectRef {
        let value = value.into();
        let value = value.parse::<f64>().unwrap();
        self.new_float(value)
    }

    pub fn new_func<S: Into<String>>(
        &self,
        name: S,
        params: Vec<String>,
        chunk: Chunk,
    ) -> ObjectRef {
        Rc::new(super::func::Func::new(name, params, chunk))
    }

    pub fn new_native_func<S: Into<String>>(
        &self,
        name: S,
        func: NativeFn,
        arity: Option<u8>,
    ) -> ObjectRef {
        Rc::new(super::native::NativeFunc::new(name, func, arity))
    }

    pub fn new_int<I: Into<BigInt>>(&self, value: I) -> ObjectRef {
        let value = value.into();
        Rc::new(super::int::Int::new(value))
    }

    pub fn new_int_from_string<S: Into<String>>(&self, value: S) -> ObjectRef {
        let value = value.into();
        let value = BigInt::from_str_radix(value.as_ref(), 10).unwrap();
        self.new_int(value)
    }

    pub fn new_string<S: Into<String>>(&self, value: S) -> ObjectRef {
        let value = value.into();
        Rc::new(super::str::Str::new(value))
    }

    pub fn new_tuple(&self, items: Vec<ObjectRef>) -> ObjectRef {
        if items.is_empty() {
            return self.empty_tuple.clone();
        }
        Rc::new(super::tuple::Tuple::new(items))
    }
}
