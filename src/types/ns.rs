use std::collections::hash_map;
use std::collections::HashMap;
use std::fmt;

use super::base::ObjectRef;

pub type NamespaceObjects = HashMap<String, ObjectRef>;

/// A namespace is a container for object attributes. Note that the
/// `Namespace` type is not a *system* type.
pub struct Namespace {
    objects: NamespaceObjects,
}

unsafe impl Send for Namespace {}
unsafe impl Sync for Namespace {}

impl Namespace {
    pub fn new() -> Self {
        Self { objects: HashMap::new() }
    }

    pub fn with_entries<S: Into<String>>(entries: Vec<(S, ObjectRef)>) -> Self {
        let mut ns = Self::new();
        for entry in entries.into_iter() {
            ns.add_entry(entry);
        }
        ns
    }

    pub fn clear(&mut self) {
        self.objects.clear()
    }

    pub fn iter(&self) -> hash_map::Iter<'_, String, ObjectRef> {
        self.objects.iter()
    }

    pub fn size(&self) -> usize {
        self.objects.len()
    }

    pub fn get_obj(&self, name: &str) -> Option<ObjectRef> {
        self.objects.get(name).cloned()
    }

    /// Add an object, settings its initial value as specified (usually
    /// nil).
    pub fn add_obj<S: Into<String>>(&mut self, name: S, obj: ObjectRef) {
        self.objects.insert(name.into(), obj);
    }

    /// This is a special case of `add_obj` that accepts an "entry"
    /// instead of a separate name and object, where an "entry" is a
    /// 2-tuple containing the name and object.
    pub fn add_entry<S: Into<String>>(&mut self, entry: (S, ObjectRef)) {
        self.objects.insert(entry.0.into(), entry.1);
    }

    /// Set an object's value. This will only succeed if the object
    /// already exists in the namespace.
    pub fn set_obj(&mut self, name: &str, obj: ObjectRef) -> bool {
        if self.objects.contains_key(name) {
            self.objects.insert(name.to_owned(), obj);
            true
        } else {
            false
        }
    }

    pub fn is_equal(&self, other: &Namespace) -> bool {
        if self.size() != other.size() {
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
