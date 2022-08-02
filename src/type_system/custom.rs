use std::any::Any;
use std::fmt;
use std::sync::Arc;

use crate::vm::RuntimeContext;

use super::base::{ObjectRef, ObjectTrait, ObjectTraitExt, TypeRef};
use super::ns::Namespace;

// Custom Object -------------------------------------------------------

pub struct Custom {
    class: ObjectRef,
    namespace: Arc<Namespace>,
}

unsafe impl Send for Custom {}
unsafe impl Sync for Custom {}

impl Custom {
    pub fn new(class: ObjectRef) -> Self {
        Self { class, namespace: Arc::new(Namespace::new()) }
    }
}

impl ObjectTrait for Custom {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        self.class.class().clone()
    }

    fn type_obj(&self) -> ObjectRef {
        self.class.clone()
    }

    fn namespace(&self) -> ObjectRef {
        self.namespace.clone()
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait, ctx: &RuntimeContext) -> bool {
        if let Some(rhs) = self.as_any().downcast_ref::<Self>() {
            if self.is(rhs) {
                return true;
            }
        }
        self.namespace.is_equal(&*rhs.namespace(), ctx)
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Custom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Check for $string attr and use that value if present
        let class = self.class();
        let id = self.id();
        write!(f, "<{class}> object @ {id}")
    }
}

impl fmt::Debug for Custom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
