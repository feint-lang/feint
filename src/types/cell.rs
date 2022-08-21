use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::RuntimeBoolResult;

use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Cell Type -----------------------------------------------------------

pub static CELL_TYPE: Lazy<Arc<RwLock<CellType>>> =
    Lazy::new(|| Arc::new(RwLock::new(CellType::new())));

pub struct CellType {
    ns: Namespace,
}

unsafe impl Send for CellType {}
unsafe impl Sync for CellType {}

impl CellType {
    pub fn new() -> Self {
        Self {
            ns: Namespace::with_entries(&[
                // Class Attributes
                ("$name", new::str("Cell")),
                ("$full_name", new::str("builtins.Cell")),
            ]),
        }
    }
}

impl TypeTrait for CellType {
    fn name(&self) -> &str {
        "Cell"
    }

    fn full_name(&self) -> &str {
        "builtins.Cell"
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }
}

impl ObjectTrait for CellType {
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

// Cell Object ---------------------------------------------------------

pub struct Cell {
    ns: Namespace,
    value: ObjectRef,
}

unsafe impl Send for Cell {}
unsafe impl Sync for Cell {}

impl Cell {
    pub fn new() -> Self {
        Self { ns: Namespace::new(), value: new::nil() }
    }

    pub fn with_value(value: ObjectRef) -> Self {
        let mut cell = Self::new();
        cell.set_value(value);
        cell
    }

    pub fn value(&self) -> ObjectRef {
        self.value.clone()
    }

    pub fn set_value(&mut self, new_value: ObjectRef) {
        self.value = new_value;
    }
}

impl ObjectTrait for Cell {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        CELL_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        CELL_TYPE.clone()
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
    }

    fn bool_val(&self) -> RuntimeBoolResult {
        Ok(false)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "&({:?})", &*self.value.read().unwrap())
    }
}

impl fmt::Debug for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
