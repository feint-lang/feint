use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Map Type ------------------------------------------------------------

pub static MAP_TYPE: Lazy<Arc<RwLock<MapType>>> =
    Lazy::new(|| Arc::new(RwLock::new(MapType::new())));

pub struct MapType {
    namespace: Namespace,
}

impl MapType {
    pub fn new() -> Self {
        Self {
            namespace: Namespace::with_entries(&[
                // Class Attributes
                ("$name", new::str("Map")),
                ("$full_name", new::str("builtins.Map")),
            ]),
        }
    }
}

unsafe impl Send for MapType {}
unsafe impl Sync for MapType {}

impl TypeTrait for MapType {
    fn name(&self) -> &str {
        "Map"
    }

    fn full_name(&self) -> &str {
        "builtins.Map"
    }

    fn ns(&self) -> &Namespace {
        &self.namespace
    }
}

impl ObjectTrait for MapType {
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
        &self.namespace
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.namespace
    }
}

// Map Object ----------------------------------------------------------

pub struct Map {
    namespace: Namespace,
    entries: RwLock<HashMap<String, ObjectRef>>,
}

impl Map {
    pub fn new(entries: HashMap<String, ObjectRef>) -> Self {
        Self { namespace: Namespace::new(), entries: RwLock::new(entries) }
    }

    pub fn len(&self) -> usize {
        let entries = self.entries.read().unwrap();
        entries.len()
    }

    pub fn is_empty(&self) -> bool {
        let entries = self.entries.read().unwrap();
        entries.is_empty()
    }

    pub fn add<S: Into<String>>(&self, key: S, val: ObjectRef) {
        let entries = &mut self.entries.write().unwrap();
        entries.insert(key.into(), val);
    }

    pub fn get(&self, name: &str) -> Option<ObjectRef> {
        let entries = self.entries.read().unwrap();
        if let Some(val) = entries.get(name) {
            Some(val.clone())
        } else {
            None
        }
    }

    pub fn to_hash_map(&self) -> HashMap<String, ObjectRef> {
        self.entries.read().unwrap().clone()
    }
}

impl ObjectTrait for Map {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn class(&self) -> TypeRef {
        MAP_TYPE.clone()
    }

    fn type_obj(&self) -> ObjectRef {
        MAP_TYPE.clone()
    }

    fn ns(&self) -> &Namespace {
        &self.namespace
    }

    fn ns_mut(&mut self) -> &mut Namespace {
        &mut self.namespace
    }

    fn is_equal(&self, rhs: &dyn ObjectTrait) -> bool {
        if let Some(rhs) = rhs.down_to_map() {
            if self.is(rhs) {
                return true;
            }
            if self.len() != rhs.len() {
                return false;
            }
            let entries = self.entries.read().unwrap();
            let rhs_entries = rhs.entries.read().unwrap();
            entries.iter().all(|(name, a_ref)| {
                if let Some(b_ref) = rhs_entries.get(name) {
                    let a = a_ref.read().unwrap();
                    let b = b_ref.read().unwrap();
                    a.is_equal(&*b)
                } else {
                    false
                }
            })
        } else {
            false
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let this_id = self.id();
        let entries = self.entries.read().unwrap();
        let entries: Vec<String> = entries
            .iter()
            .map(|(name, val)| {
                let val = val.read().unwrap();
                if val.id() == this_id {
                    "{...}".to_owned()
                } else {
                    format!("{name:?} => {:?}", &*val)
                }
            })
            .collect();
        let string = entries.join(", ");
        write!(f, "{{{string}}}")
    }
}

impl fmt::Debug for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
