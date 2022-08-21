use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::new;
use super::result::Params;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::func_trait::FuncTrait;
use super::ns::Namespace;

// Closure Type --------------------------------------------------------

pub static CLOSURE_TYPE: Lazy<Arc<RwLock<ClosureType>>> =
    Lazy::new(|| Arc::new(RwLock::new(ClosureType::new())));

pub struct ClosureType {
    ns: Namespace,
}

unsafe impl Send for ClosureType {}
unsafe impl Sync for ClosureType {}

impl ClosureType {
    pub fn new() -> Self {
        Self {
            ns: Namespace::with_entries(&[
                // Class Attributes
                ("$name", new::str("Closure")),
                ("$full_name", new::str("builtins.Closure")),
            ]),
        }
    }
}

impl TypeTrait for ClosureType {
    fn name(&self) -> &str {
        "Closure"
    }

    fn full_name(&self) -> &str {
        "builtins.Closure"
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }
}

impl ObjectTrait for ClosureType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn class(&self) -> TypeRef {
        TYPE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        TYPE_TYPE.clone()
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
    }
}

// Closure Object ------------------------------------------------------

pub struct Closure {
    ns: Namespace,
    name: String,
    params: Params,
    pub func: ObjectRef,
    pub captured: HashMap<String, ObjectRef>,
}

unsafe impl Send for Closure {}
unsafe impl Sync for Closure {}

impl Closure {
    pub fn new(func_ref: ObjectRef, captured: HashMap<String, ObjectRef>) -> Self {
        let func = func_ref.read().unwrap();
        let func = func.down_to_func().unwrap();
        Self {
            ns: Namespace::new(),
            name: func.name().to_owned(),
            params: func.params().clone(),
            func: func_ref.clone(),
            captured,
        }
    }
}

impl FuncTrait for Closure {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn params(&self) -> &Params {
        &self.params
    }
}

impl ObjectTrait for Closure {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn class(&self) -> TypeRef {
        CLOSURE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        CLOSURE_TYPE.clone()
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[closure] {}", self.func.read().unwrap())
    }
}

impl fmt::Debug for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
