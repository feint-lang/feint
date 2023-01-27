use std::fmt;

use indexmap::IndexMap;

use super::base::ObjectRef;
use super::map::Map;

pub type Objects = IndexMap<String, ObjectRef>;

/// A namespace is a container for object attributes. Note that the
/// `Namespace` type is not a *system* type.
pub struct Namespace {
    objects: Objects,
}

unsafe impl Send for Namespace {}
unsafe impl Sync for Namespace {}

impl Default for Namespace {
    fn default() -> Self {
        Self::new(IndexMap::default())
    }
}

impl Namespace {
    pub fn new(objects: Objects) -> Self {
        Self { objects }
    }

    pub fn with_entries(entries: &[(&str, ObjectRef)]) -> Self {
        let mut ns = Self::default();
        ns.extend(entries);
        ns
    }

    pub fn clear(&mut self) {
        self.objects.clear()
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.objects.contains_key(name)
    }

    pub fn iter(&self) -> indexmap::map::Iter<'_, String, ObjectRef> {
        self.objects.iter()
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn get(&self, name: &str) -> Option<ObjectRef> {
        self.objects.get(name).cloned()
    }

    /// Add an object, settings its initial value as specified (usually
    /// nil).
    pub fn insert<S: Into<String>>(&mut self, name: S, obj: ObjectRef) {
        self.objects.insert(name.into(), obj);
    }

    /// Set an object's value. This will only succeed if the object
    /// already exists in the namespace.
    pub fn set(&mut self, name: &str, obj: ObjectRef) -> bool {
        if self.objects.contains_key(name) {
            self.objects.insert(name.to_owned(), obj);
            true
        } else {
            false
        }
    }

    pub fn extend(&mut self, entries: &[(&str, ObjectRef)]) {
        self.objects.extend(entries.iter().map(|(k, v)| (k.to_string(), v.clone())));
    }

    pub fn extend_from_map(&mut self, map: &Map) {
        for (name, val) in map.entries().read().unwrap().iter() {
            self.insert(name, val.clone());
        }
    }

    pub fn is_equal(&self, other: &Namespace) -> bool {
        if self.len() != other.len() {
            // Namespaces have a different number of entries, so
            // they can't be equal.
            return false;
        }
        if !self.objects.keys().all(|k| other.objects.contains_key(k)) {
            // Namespaces have differing keys, so they can't be
            // equal.
            return false;
        }
        // Otherwise, compare all entries for equality.
        for (name, lhs_val) in self.objects.iter() {
            let lhs_val = lhs_val.read().unwrap();
            let rhs_val = &other.objects[name].read().unwrap();
            if !lhs_val.is_equal(&**rhs_val) {
                return false;
            }
        }
        true
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<namespace>")
    }
}

impl fmt::Debug for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
