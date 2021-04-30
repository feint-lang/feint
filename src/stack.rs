use std::cmp::min;

pub struct Stack<T> {
    storage: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack {
            storage: Vec::new(),
        }
    }

    pub fn push(&mut self, item: T) {
        self.storage.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.storage.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        match self.size() {
            0 => None,
            _ => self.storage.last(),
        }
    }

    /// Get the top N items from the stack. If the stack contains less
    /// than N items, get all the items it does contain.
    pub fn peek_n(&self, n: u8) -> &[T] {
        let size = self.size();
        let start = size - min(n as usize, size);
        &self.storage[start..size]
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    pub fn size(&self) -> usize {
        self.storage.len()
    }

    pub fn clear(&mut self) {
        self.storage.clear()
    }
}
