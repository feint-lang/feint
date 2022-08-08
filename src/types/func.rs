use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::Code;

use super::create;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;
use super::result::Params;

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
    pub arity: Option<usize>,
    pub code: Code,
}

unsafe impl Send for Func {}
unsafe impl Sync for Func {}

impl Func {
    pub fn new<S: Into<String>>(name: S, params: Params, code: Code) -> Self {
        let name = name.into();
        let arity = params.as_ref().map(|params| params.len());
        let arity_obj = arity.map_or_else(create::new_nil, |len| create::new_int(len));
        Self {
            namespace: Namespace::with_entries(vec![
                // Instance Attributes
                ("$name", create::new_str(name.as_str())),
                ("$arity", arity_obj),
            ]),
            name,
            params,
            arity,
            code,
        }
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
        let num_args = match self.arity {
            Some(n) => n.to_string(),
            None => "...".to_string(),
        };
        let id = self.id();
        write!(f, "function {name}/{num_args} @ {id}")
    }
}

impl fmt::Debug for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
