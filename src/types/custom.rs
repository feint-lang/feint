use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::module::Module;
use super::ns::Namespace;

// Custom Type ---------------------------------------------------------

pub struct CustomType {
    ns: Namespace,
    module: new::obj_ref_t!(Module),
    name: String,
    full_name: String,
}

impl CustomType {
    pub fn new(module: new::obj_ref_t!(Module), name: String) -> Self {
        let full_name = format!("{}.{name}", module.read().unwrap().name());
        Self {
            ns: Namespace::with_entries(&[
                // Class Attributes
                ("$name", new::str(name.as_str())),
                ("$full_name", new::str(full_name.as_str())),
            ]),
            module,
            name,
            full_name,
        }
    }
}

gen::standard_object_impls!(CustomType);

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

impl ObjectTrait for CustomType {
    gen::object_trait_header!(TYPE_TYPE);
}

// Custom Object -------------------------------------------------------

pub struct CustomObj {
    class: new::obj_ref_t!(CustomType),
    ns: Namespace,
}

gen::standard_object_impls!(CustomObj);

impl CustomObj {
    pub fn new(class: new::obj_ref_t!(CustomType), attrs: Namespace) -> Self {
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

    fn type_obj(&self) -> ObjectRef {
        self.class.clone()
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
        self.ns.set_obj(name, value);
        new::nil()
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        self.is(rhs) || rhs.is_always() || self.ns.is_equal(rhs.ns())
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for CustomObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Check for $string attr and use that value if present
        let class = self.class.read().unwrap();
        write!(f, "<{}> object @ {}", class.full_name(), self.id())
    }
}

impl fmt::Debug for CustomObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
