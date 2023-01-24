use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::RuntimeBoolResult;

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Cell Type -----------------------------------------------------------

gen::type_and_impls!(CellType, Cell);

pub static CELL_TYPE: Lazy<gen::obj_ref_t!(CellType)> =
    Lazy::new(|| gen::obj_ref!(CellType::new()));

// Cell Object ---------------------------------------------------------

pub struct Cell {
    ns: Namespace,
    value: ObjectRef,
}

gen::standard_object_impls!(Cell);

impl Cell {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { ns: Namespace::default(), value: new::nil() }
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
    gen::object_trait_header!(CELL_TYPE);

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
