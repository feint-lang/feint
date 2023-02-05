//! Builtin `CustomType` type.
use std::any::Any;
use std::fmt;

use feint_code_gen::*;

use crate::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef};
use super::ns::Namespace;

pub struct CustomObj {
    class: TypeRef,
    ns: Namespace,
}

standard_object_impls!(CustomObj);

impl CustomObj {
    pub fn new(class: TypeRef, attrs: Namespace) -> Self {
        Self { class, ns: attrs }
    }
}

impl ObjectTrait for CustomObj {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        self.class.clone()
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
    }

    fn set_attr(
        &mut self,
        name: &str,
        value: ObjectRef,
        _this: ObjectRef,
    ) -> ObjectRef {
        self.ns.set(name, value);
        new::nil()
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        self.is(rhs) || rhs.is_always() || self.ns.is_equal(rhs.ns())
    }
}

impl fmt::Display for CustomObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let class = self.class();
        let class = class.read().unwrap();
        write!(f, "<{} object>", class.name())
    }
}

impl fmt::Debug for CustomObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let class = self.class();
        let class = class.read().unwrap();
        write!(f, "<{} object @ {}>", class.full_name(), self.id())
    }
}
