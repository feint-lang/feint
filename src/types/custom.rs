use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use super::new;
use super::result::SetAttrResult;

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

unsafe impl Send for CustomType {}
unsafe impl Sync for CustomType {}

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

// Custom Object -------------------------------------------------------

pub struct CustomObj {
    class: new::obj_ref_t!(CustomType),
    ns: Namespace,
}

unsafe impl Send for CustomObj {}
unsafe impl Sync for CustomObj {}

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

    fn set_attr(&mut self, name: &str, value: ObjectRef) -> SetAttrResult {
        self.ns.set_obj(name, value);
        Ok(())
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if self.is(rhs) {
            return true;
        }
        self.ns.is_equal(rhs.ns())
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
