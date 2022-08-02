use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::vm::RuntimeContext;
use once_cell::sync::Lazy;

use super::create;

use super::base::{ObjectRef, ObjectTrait, ObjectTraitExt, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;

pub type NamespaceObjects = HashMap<String, ObjectRef>;

// NS Type -------------------------------------------------------------

pub static NS_TYPE: Lazy<Arc<NamespaceType>> =
    Lazy::new(|| Arc::new(NamespaceType::new()));

pub struct NamespaceType {
    namespace: Arc<Namespace>,
}

impl NamespaceType {
    pub fn new() -> Self {
        let mut ns = Namespace::new();
        ns.add_obj("$name", create::new_str("Namespace"));
        ns.add_obj("$full_name", create::new_str("builtins.Namespace"));
        Self { namespace: Arc::new(ns) }
    }
}

unsafe impl Send for NamespaceType {}
unsafe impl Sync for NamespaceType {}

impl TypeTrait for NamespaceType {
    fn name(&self) -> &str {
        "Namespace"
    }

    fn full_name(&self) -> &str {
        "builtins.Namespace"
    }
}

impl ObjectTrait for NamespaceType {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        TYPE_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        TYPE_TYPE.clone()
    }

    fn namespace(&self) -> ObjectRef {
        self.namespace.clone()
    }
}

// NS Object -----------------------------------------------------------

pub struct Namespace {
    objects: NamespaceObjects,
}

unsafe impl Send for Namespace {}
unsafe impl Sync for Namespace {}

impl Namespace {
    pub fn new() -> Self {
        Self { objects: HashMap::new() }
    }

    pub fn size(&self) -> usize {
        self.objects.len()
    }

    pub fn get_obj(&self, name: &str) -> Option<ObjectRef> {
        if let Some(obj) = self.objects.get(name) {
            Some(obj.clone())
        } else {
            None
        }
    }

    pub fn add_obj<S: Into<String>>(&mut self, name: S, obj: ObjectRef) {
        self.objects.insert(name.into(), obj);
    }
}

impl ObjectTrait for Namespace {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn class(&self) -> TypeRef {
        NS_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        NS_TYPE.clone()
    }

    // XXX: This is a bit of a hack due to avoid a circularity. The
    //      return value should NOT be used.
    fn namespace(&self) -> ObjectRef {
        create::new_namespace()
    }

    // fn get_attr(&self, _name: &str) -> Option<ObjectRef> {
    //     panic!("Don't use Namespace::get_attr()");
    // }

    fn is_equal(&self, rhs: &dyn ObjectTrait, ctx: &RuntimeContext) -> bool {
        if let Some(rhs) = rhs.down_to_ns() {
            if self.is(rhs) {
                return true;
            }
            if self.size() != rhs.size() {
                // Namespaces have a different number of entries, so
                // they can't be equal.
                return false;
            }
            if !self.objects.keys().all(|k| rhs.objects.contains_key(k)) {
                // Namespaces have differing keys, so they can't be
                // equal.
                return false;
            }
            // Otherwise, compare all entries for equality.
            for (name, lhs_val) in self.objects.iter() {
                let rhs_val = &rhs.objects[name];
                if !lhs_val.is_equal(&**rhs_val, ctx) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.type_obj(), self.id())
    }
}

impl fmt::Debug for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

#[cfg(test)]
mod tests {
    use super::super::base::ObjectTraitExt;
    use super::*;

    #[test]
    fn make_ns() {
        let ns = Namespace::new();

        let class = ns.get_attr("$type").unwrap();
        assert!(class.is(&*NS_TYPE.clone()));

        let class_type = class.get_attr("$type").unwrap();
        assert!(class_type.is(&*TYPE_TYPE.clone()));

        let class_type_type = class_type.get_attr("$type").unwrap();
        assert!(class_type_type.is(&*TYPE_TYPE.clone()));

        let module = ns.get_attr("$module").unwrap();
        assert_eq!(module.down_to_mod().unwrap().name(), "builtins");

        let name = ns.get_attr("$name").unwrap();
        assert_eq!(name.down_to_str().unwrap().value(), "Namespace");

        let full_name = ns.get_attr("$full_name").unwrap();
        assert_eq!(full_name.down_to_str().unwrap().value(), "builtins.Namespace");

        let id = ns.get_attr("$id");
        assert!(id.is_some());
        assert!(id.unwrap().down_to_int().is_some());
    }
}
