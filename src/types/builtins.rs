use std::sync::Arc;

use num_bigint::BigInt;
use num_traits::Num;

use crate::vm::Chunk;

use super::builtin_func::BuiltinFn;
use super::class::{Type, TypeRef};
use super::custom::Custom;
use super::object::ObjectRef;
use super::result::Params;

pub struct Builtins {
    // Singletons
    pub nil_obj: Arc<super::nil::Nil>,
    pub true_obj: Arc<super::bool::Bool>,
    pub false_obj: Arc<super::bool::Bool>,
    pub empty_tuple: Arc<super::tuple::Tuple>,
}

impl Builtins {
    pub fn new() -> Self {
        // Singletons
        let nil_obj = Arc::new(super::nil::Nil::new());
        let true_obj = Arc::new(super::bool::Bool::new(true));
        let false_obj = Arc::new(super::bool::Bool::new(false));
        let empty_tuple = Arc::new(super::tuple::Tuple::new(vec![]));
        Self { nil_obj, true_obj, false_obj, empty_tuple }
    }

    pub fn new_type(&self, module: &str, name: &str) -> TypeRef {
        let class = Type::new(module, name);
        Arc::new(class)
    }

    // Builtin type constructors ---------------------------------------

    pub fn new_builtin_func<S: Into<String>>(
        &self,
        name: S,
        params: Option<Vec<S>>,
        func: BuiltinFn,
    ) -> ObjectRef {
        let params = self.collect_params(params);
        Arc::new(super::builtin_func::BuiltinFunc::new(name, params, func))
    }

    pub fn new_float<F: Into<f64>>(&self, value: F) -> ObjectRef {
        let value = value.into();
        Arc::new(super::float::Float::new(value))
    }

    pub fn new_float_from_string<S: Into<String>>(&self, value: S) -> ObjectRef {
        let value = value.into();
        let value = value.parse::<f64>().unwrap();
        self.new_float(value)
    }

    pub fn new_func<S: Into<String>>(
        &self,
        name: S,
        params: Option<Vec<S>>,
        chunk: Chunk,
    ) -> ObjectRef {
        let params = self.collect_params(params);
        Arc::new(super::func::Func::new(name, params, chunk))
    }

    pub fn new_int<I: Into<BigInt>>(&self, value: I) -> ObjectRef {
        let value = value.into();
        Arc::new(super::int::Int::new(value))
    }

    pub fn new_namespace(&self, name: &str) -> ObjectRef {
        let ns = super::namespace::Namespace::new(name, self.nil_obj.clone());
        Arc::new(ns)
    }

    pub fn new_int_from_string<S: Into<String>>(&self, value: S) -> ObjectRef {
        let value = value.into();
        let value = BigInt::from_str_radix(value.as_ref(), 10).unwrap();
        self.new_int(value)
    }

    pub fn new_str<S: Into<String>>(&self, value: S) -> ObjectRef {
        let value = value.into();
        Arc::new(super::str::Str::new(value))
    }

    pub fn new_tuple(&self, items: Vec<ObjectRef>) -> ObjectRef {
        if items.is_empty() {
            return self.empty_tuple.clone();
        }
        Arc::new(super::tuple::Tuple::new(items))
    }

    // Custom type constructor -----------------------------------------

    pub fn new_custom_instance(&self, class: TypeRef) -> ObjectRef {
        Arc::new(Custom::new(class))
    }

    // Utilities -------------------------------------------------------

    /// Convert Rust bool to builtin Bool object
    pub fn bool_obj_from_bool(&self, value: bool) -> ObjectRef {
        if value {
            self.true_obj.clone()
        } else {
            self.false_obj.clone()
        }
    }

    /// Collect parameters for function types.
    fn collect_params<S: Into<String>>(&self, params: Option<Vec<S>>) -> Params {
        if let Some(names) = params {
            let mut params = vec![];
            for name in names {
                params.push(name.into());
            }
            Some(params)
        } else {
            None
        }
    }
}
