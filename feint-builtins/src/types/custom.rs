use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use super::new;
use feint_code_gen::*;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Custom Type ---------------------------------------------------------

/// TODO: This shouldn't need to be cloneable
#[derive(Clone)]
pub struct CustomType {
    ns: Namespace,
    module: ObjectRef,
    name: String,
    full_name: String,
}

impl CustomType {
    pub fn new(module_ref: ObjectRef, name: String) -> Self {
        let module = module_ref.read().unwrap();
        let module = module.down_to_mod().unwrap();
        let full_name = format!("{}.{name}", module.name());
        let type_ref = Self {
            ns: Namespace::with_entries(&[
                // Class Attributes
                ("$module_name", new::str(module.name())),
                ("$full_name", new::str(&full_name)),
                ("$name", new::str(&name)),
            ]),
            module: module_ref.clone(),
            name,
            full_name,
        };
        type_ref
    }
}

standard_object_impls!(CustomType);

impl TypeTrait for CustomType {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn full_name(&self) -> &str {
        self.full_name.as_str()
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn module(&self) -> ObjectRef {
        self.module.clone()
    }
}

/// NOTE: This is customized so the module is correct.
impl ObjectTrait for CustomType {
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

    fn as_type(&self) -> Option<&dyn TypeTrait> {
        Some(self)
    }

    fn module(&self) -> ObjectRef {
        self.module.clone()
    }
}

// Custom Object -------------------------------------------------------

pub struct CustomObj {
    type_obj: obj_ref_t!(CustomType),
    ns: Namespace,
}

standard_object_impls!(CustomObj);

impl CustomObj {
    pub fn new(type_obj: obj_ref_t!(CustomType), attrs: Namespace) -> Self {
        Self { type_obj, ns: attrs }
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
        self.type_obj.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        self.type_obj.clone()
    }

    fn ns(&self) -> &Namespace {
        &self.ns
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns
    }

    fn as_type(&self) -> Option<&dyn TypeTrait> {
        None
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

// Display -------------------------------------------------------------

impl fmt::Display for CustomObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let class = self.class();
        let class = class.read().unwrap();
        write!(f, "<{} object @ {}>", class.full_name(), self.id())
    }
}

impl fmt::Debug for CustomObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
