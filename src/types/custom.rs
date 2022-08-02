use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::sync::Arc;

use super::create;
use super::result::SetAttrResult;

use super::base::{ObjectRef, ObjectTrait, ObjectTraitExt, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::module::Module;
use super::ns::Namespace;

// Custom Type ---------------------------------------------------------

pub struct CustomType {
    namespace: RefCell<Namespace>,
    module: Arc<Module>,
    name: String,
}

impl CustomType {
    pub fn new<S: Into<String>>(module: Arc<Module>, name: S) -> Self {
        let mut ns = Namespace::new();
        let name = name.into();
        ns.add_obj("$name", create::new_str(name.as_str()));
        ns.add_obj("$full_name", create::new_str(module.name()));
        Self { namespace: RefCell::new(ns), module, name }
    }
}

unsafe impl Send for CustomType {}
unsafe impl Sync for CustomType {}

impl TypeTrait for CustomType {
    fn module(&self) -> ObjectRef {
        self.module.clone()
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn full_name(&self) -> &str {
        self.module.name()
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

    fn namespace(&self) -> &RefCell<Namespace> {
        &self.namespace
    }
}

// Custom Object -------------------------------------------------------

pub struct CustomObj {
    class: Arc<CustomType>,
    namespace: RefCell<Namespace>,
}

unsafe impl Send for CustomObj {}
unsafe impl Sync for CustomObj {}

impl CustomObj {
    pub fn new(class: Arc<CustomType>, attrs: Namespace) -> Self {
        Self { class, namespace: RefCell::new(attrs) }
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

    fn namespace(&self) -> &RefCell<Namespace> {
        &self.namespace
    }

    fn set_attr(&mut self, name: &str, value: ObjectRef) -> SetAttrResult {
        self.namespace.borrow_mut().set_obj(name, value);
        Ok(())
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            if self.is(rhs) {
                return true;
            }
        }
        self.namespace.borrow().is_equal(&rhs.namespace().borrow())
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for CustomObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Check for $string attr and use that value if present
        write!(f, "<{}> object @ {}", self.class.full_name(), self.id())
    }
}

impl fmt::Debug for CustomObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
