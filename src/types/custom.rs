use std::any::Any;
use std::fmt;
use std::sync::{Arc, RwLock};

use super::create;
use super::result::SetAttrResult;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::module::Module;
use super::ns::Namespace;

// Custom Type ---------------------------------------------------------

pub struct CustomType {
    namespace: Namespace,
    module: Arc<RwLock<Module>>,
    name: String,
    full_name: String,
}

impl CustomType {
    pub fn new(module: Arc<RwLock<Module>>, name: String) -> Self {
        let full_name = format!("{}.{name}", module.read().unwrap().name());
        Self {
            namespace: Namespace::with_entries(&[
                // Class Attributes
                ("$name", create::new_str(name.as_str())),
                ("$full_name", create::new_str(full_name.as_str())),
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

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    fn module(&self) -> ObjectRef {
        self.module.clone()
    }
}

impl ObjectTrait for CustomType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        TYPE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        TYPE_TYPE.clone()
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }
}

// Custom Object -------------------------------------------------------

pub struct CustomObj {
    class: Arc<RwLock<CustomType>>,
    namespace: Namespace,
}

unsafe impl Send for CustomObj {}
unsafe impl Sync for CustomObj {}

impl CustomObj {
    pub fn new(class: Arc<RwLock<CustomType>>, attrs: Namespace) -> Self {
        Self { class, namespace: attrs }
    }
}

impl ObjectTrait for CustomObj {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        self.class.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        self.class.clone()
    }

    fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    fn set_attr(&mut self, name: &str, value: ObjectRef) -> SetAttrResult {
        self.namespace.set_obj(name, value);
        Ok(())
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if self.is(rhs) {
            return true;
        }
        self.namespace.is_equal(rhs.namespace())
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
