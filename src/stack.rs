pub struct Stack<T> {
    storage: Vec<T>
}

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack{ storage: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.storage.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.storage.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }
}
