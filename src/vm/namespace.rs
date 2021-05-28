use std::collections::HashMap;

#[derive(Debug)]
pub struct Namespace {
    storage: HashMap<String, usize>,
}

impl Namespace {
    pub fn new() -> Self {
        Namespace { storage: HashMap::new() }
    }

    pub fn add<S: Into<String>>(&mut self, key: S, const_index: usize) {
        self.storage.insert(key.into(), const_index);
    }

    pub fn get(&self, name: &str) -> Option<&usize> {
        self.storage.get(name)
    }
}
