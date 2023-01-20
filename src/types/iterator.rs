use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// IteratorType Type ---------------------------------------------------

gen::type_and_impls!(IteratorType, Iterator);

pub static ITERATOR_TYPE: Lazy<gen::obj_ref_t!(IteratorType)> = Lazy::new(|| {
    let type_ref = gen::obj_ref!(IteratorType::new());
    let mut type_obj = type_ref.write().unwrap();

    type_obj.add_attrs(&[
        // Instance Attributes -----------------------------------------
        gen::meth!("next", type_ref, &[], "", |this, _, _| {
            let mut this = this.write().unwrap();
            let this = this.down_to_iterator_mut().unwrap();
            Ok(this.next())
        }),
    ]);

    type_ref.clone()
});

// Iterator Object -----------------------------------------------------

pub struct FIIterator {
    ns: Namespace,
    wrapped: Vec<ObjectRef>,
    current: usize,
}

gen::standard_object_impls!(FIIterator);

impl FIIterator {
    pub fn new(wrapped: Vec<ObjectRef>) -> Self {
        Self { ns: Namespace::new(), wrapped, current: 0 }
    }

    fn next(&mut self) -> ObjectRef {
        if self.current < self.wrapped.len() {
            let obj = self.wrapped.get(self.current).unwrap();
            self.current += 1;
            obj.clone()
        } else {
            new::nil()
        }
    }
}

impl ObjectTrait for FIIterator {
    gen::object_trait_header!(ITERATOR_TYPE);
}

// Display -------------------------------------------------------------

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
