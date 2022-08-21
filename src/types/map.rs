use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use super::gen;
use super::new;

use super::base::{ObjectRef, ObjectTrait, TypeRef, TypeTrait};
use super::class::TYPE_TYPE;
use super::ns::Namespace;

// Map Type ------------------------------------------------------------

gen::type_and_impls!(MapType, Map);

pub static MAP_TYPE: Lazy<new::obj_ref_t!(MapType)> =
    Lazy::new(|| new::obj_ref!(MapType::new()));

// Map Object ----------------------------------------------------------

pub struct Map {
    ns: Namespace,
    entries: RwLock<HashMap<String, ObjectRef>>,
}

gen::standard_object_impls!(Map);

impl Map {
    pub fn new(entries: HashMap<String, ObjectRef>) -> Self {
        Self { ns: Namespace::new(), entries: RwLock::new(entries) }
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
    gen::object_trait_header!(MAP_TYPE);

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
