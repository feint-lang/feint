use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use feint_code_gen::*;

use crate::BUILTINS;

use super::base::{ObjectRef, ObjectTrait, TypeRef};

use super::ns::Namespace;

// Custom Type ---------------------------------------------------------

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
    object_trait_header!();

    fn set_attr(
        &mut self,
        name: &str,
        value: ObjectRef,
        _this: ObjectRef,
    ) -> ObjectRef {
        self.ns.set(name, value);
        BUILTINS.nil()
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        self.is(rhs) || rhs.is_always() || self.ns.is_equal(rhs.ns())
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for CustomObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let class = self.class();
        // let class = class.read().unwrap();
        // write!(f, "<{} object @ {}>", class.full_name(), self.id())
        write!(f, "<object @ {}>", self.id())
    }
}

impl fmt::Debug for CustomObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
