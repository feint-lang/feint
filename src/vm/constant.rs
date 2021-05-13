pub struct Constant {
    pub value: usize,
}

impl Constant {
    pub fn new(value: usize) -> Self {
        Self { value }
    }
}

pub struct ConstantStore {
    storage: Vec<Constant>,
}

impl ConstantStore {
    pub fn new() -> Self {
        Self { storage: Vec::new() }
    }

    pub fn add(&mut self, constant: Constant) -> usize {
        let index = self.storage.len();
        self.storage.push(constant);
        return index;
    }

    pub fn get(&self, index: usize) -> Option<&Constant> {
        self.storage.get(index)
    }
}
