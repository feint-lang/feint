use std::collections::HashMap;

pub struct Namespace {
    parent: Option<Box<Namespace>>,
    storage: HashMap<String, usize>,
    assigned_names: Vec<String>,
}

impl Namespace {
    pub fn new(parent: Option<Box<Namespace>>) -> Self {
        Namespace {
            parent,
            storage: HashMap::new(),
            assigned_names: vec![],
        }
    }

    pub fn get(&self, name: String) -> Option<usize> {
        match self.storage.get(name.as_str()) {
            Some(value) => Some(*value),
            None => {
                if self.parent.is_some() {
                    self.parent.as_ref().unwrap().get(name)
                } else {
                    None
                }
            }
        }
    }

    pub fn insert(&mut self, key: String, value: usize) -> Option<usize> {
        self.storage.insert(key, value)
    }

    pub fn assign(&mut self, name: String, value: usize) -> Result<Option<usize>, String> {
        match self.storage.contains_key(name.as_str()) {
            true => Err(format!("Cannot re-assign {}", name)),
            false => {
                self.assigned_names.push(name.clone());
                Ok(self.storage.insert(name, value))
            },
        }
    }

    pub fn reset(&mut self) {
        for name in self.assigned_names.iter() {
            self.storage.remove(name);
        }
    }
}

impl Default for Namespace {
    fn default() -> Self {
        let mut namespace = Namespace::new(None);
        namespace.insert("print".to_string(), 0);
        namespace
    }
}
