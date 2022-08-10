use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::Code;

use super::create;
use super::result::Params;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Function Type -------------------------------------------------------

pub static FUNC_TYPE: Lazy<Arc<RwLock<FuncType>>> =
    Lazy::new(|| Arc::new(RwLock::new(FuncType::new())));

pub struct FuncType {
    namespace: Namespace,
}

unsafe impl Send for FuncType {}
unsafe impl Sync for FuncType {}

impl FuncType {
    pub fn new() -> Self {
        Self {
            namespace: Namespace::with_entries(vec![
                // Class Attributes
                ("$name", create::new_str("Func")),
                ("$full_name", create::new_str("builtins.Func")),
            ]),
        }
    }
}

impl TypeTrait for FuncType {
    fn name(&self) -> &str {
        "Func"
    }

    fn full_name(&self) -> &str {
        "builtins.Func"
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

impl ObjectTrait for FuncType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        TYPE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        TYPE_TYPE.clone()
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

// Func Object ----------------------------------------------------------

pub struct Func {
    namespace: Namespace,
    pub name: String,
    pub params: Params,
    pub code: Code,
}

unsafe impl Send for Func {}
unsafe impl Sync for Func {}

impl Func {
    pub fn new<S: Into<String>>(name: S, params: Params, code: Code) -> Self {
        let name = name.into();
        Self {
            namespace: Namespace::with_entries(vec![
                // Instance Attributes
                ("$name", create::new_str(name.as_str())),
            ]),
            name,
            params,
            code,
        }
    }

    pub fn arity(&self) -> usize {
        if let Some(name) = self.params.last() {
            if name.is_empty() {
                // Has var args; return number of required args
                self.params.len() - 1
            } else {
                // Does not have var args; all args required
                self.params.len()
            }
        } else {
            0
        }
    }

    pub fn var_args_index(&self) -> Option<usize> {
        if let Some(name) = self.params.last() {
            if name.is_empty() {
                return Some(self.params.len() - 1);
            }
        }
        None
    }
}

impl ObjectTrait for Func {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        FUNC_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        FUNC_TYPE.clone()
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = &self.name;
        let arity = self.arity();
        let suffix = if self.var_args_index().is_some() { "+" } else { "" };
        let id = self.id();
        write!(f, "function {name}/{arity}{suffix} @ {id}")
    }
}

impl fmt::Debug for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
