use std::collections::HashMap;

pub struct Namespace {
    parent: Option<Box<Namespace>>,
    storage: HashMap<String, usize>,
}

impl Namespace {
    pub fn new(parent: Option<Box<Namespace>>) -> Self {
        Namespace { parent, storage: HashMap::new() }
    }

    pub fn get(&self, name: &str) -> Option<&usize> {
        if let Some(value) = self.storage.get(name) {
            Some(value)
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }

    pub fn add<S: Into<String>>(&mut self, key: S, const_index: usize) {
        self.storage.insert(key.into(), const_index);
    }

    pub fn reset(&mut self) {
        self.storage.clear();
    }
}

impl Default for Namespace {
    fn default() -> Self {
        Namespace::new(None)
    }
}
