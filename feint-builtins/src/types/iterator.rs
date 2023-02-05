//! Builtin `Iterator` type.
use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use feint_code_gen::*;

use crate::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef};
use super::class::Type;
use super::ns::Namespace;

pub static ITERATOR_TYPE: Lazy<TypeRef> = Lazy::new(|| {
    let type_ref = obj_ref!(Type::new("std", "Iterator"));

    {
        type_ref.write().unwrap().ns_mut().extend(&[
            // Instance Methods ----------------------------------------
            meth!("next", type_ref, &[], "", |this, _| {
                let mut this = this.write().unwrap();
                let this = this.down_to_iterator_mut().unwrap();
                this.next()
            }),
            meth!("peek", type_ref, &[], "", |this, _| {
                let this = this.write().unwrap();
                let this = this.down_to_iterator().unwrap();
                this.peek()
            }),
        ]);
    }

    type_ref
});

pub struct FIIterator {
    ns: Namespace,
    wrapped: Vec<ObjectRef>,
    current: usize,
}

standard_object_impls!(FIIterator);

impl FIIterator {
    pub fn new(wrapped: Vec<ObjectRef>) -> Self {
        Self { ns: Namespace::default(), wrapped, current: 0 }
    }

    fn next(&mut self) -> ObjectRef {
        let obj = self.get_or_nil(self.current);
        if self.current < self.len() {
            self.current += 1;
        }
        obj
    }

    fn peek(&self) -> ObjectRef {
        self.get_or_nil(self.current)
    }

    fn len(&self) -> usize {
        self.wrapped.len()
    }

    fn get_or_nil(&self, index: usize) -> ObjectRef {
        if index >= self.len() {
            new::nil()
        } else {
            self.wrapped[index].clone()
        }
    }
}

impl ObjectTrait for FIIterator {
    object_trait_header!(ITERATOR_TYPE);
}

impl fmt::Display for FIIterator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<iterator>")
    }
}

impl fmt::Debug for FIIterator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
