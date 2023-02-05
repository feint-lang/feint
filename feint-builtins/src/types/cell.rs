use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use feint_code_gen::*;

use crate::BUILTINS;

use super::base::{ObjectRef, ObjectTrait, TypeRef};

use super::ns::Namespace;

// Cell ---------------------------------------------------------

pub struct Cell {
    class: TypeRef,
    ns: Namespace,
    value: ObjectRef,
}

standard_object_impls!(Cell);

impl Cell {
    pub fn new(class: TypeRef) -> Self {
        Self { class, ns: Namespace::default(), value: BUILTINS.nil() }
    }

    pub fn with_value(class: TypeRef, value: ObjectRef) -> Self {
        let mut cell = Self::new(class);
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
    object_trait_header!();

    fn bool_val(&self) -> Option<bool> {
        Some(false)
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
