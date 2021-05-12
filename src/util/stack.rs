use std::cmp::min;

pub struct Stack<T> {
    storage: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack { storage: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.storage.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.storage.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.storage.last()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stack_is_empty() {
        let stack: Stack<usize> = Stack::new();
        assert_eq!(stack.is_empty(), true);
    }

    #[test]
    fn push() {
        let mut stack: Stack<usize> = Stack::new();
        assert_eq!(stack.size(), 0);
        stack.push(0);
        assert_eq!(stack.size(), 1);
    }

    #[test]
    fn pop_empty() {
        let mut stack: Stack<usize> = Stack::new();
        assert_eq!(stack.size(), 0);
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.size(), 0);
    }

    #[test]
    fn pop() {
        let mut stack: Stack<usize> = Stack::new();
        stack.push(1);
        assert_eq!(stack.size(), 1);
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.size(), 0);
    }

    #[test]
    fn peek_empty() {
        let stack: Stack<usize> = Stack::new();
        assert_eq!(stack.size(), 0);
        assert_eq!(stack.peek(), None);
        assert_eq!(stack.size(), 0);
    }

    #[test]
    fn peek() {
        let mut stack: Stack<usize> = Stack::new();
        stack.push(1);
        assert_eq!(stack.size(), 1);
        assert_eq!(stack.peek(), Some(&1));
        assert_eq!(stack.size(), 1);
    }

    #[test]
    fn clear() {
        let mut stack: Stack<usize> = Stack::new();
        assert_eq!(stack.size(), 0);
        stack.push(1);
        stack.push(2);
        assert_eq!(stack.size(), 2);
        stack.clear();
        assert_eq!(stack.size(), 0);
    }
}
