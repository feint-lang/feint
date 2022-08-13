use core::iter::Rev;
use core::slice::Iter;
use std::fmt;
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Stack<T> {
    storage: Vec<T>,
}

impl<T> Index<usize> for Stack<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.storage[index]
    }
}

impl<T> IndexMut<usize> for Stack<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.storage[index]
    }
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack { storage: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Stack { storage: Vec::with_capacity(capacity) }
    }

    pub fn push(&mut self, item: T) {
        self.storage.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.storage.pop()
    }

    /// Pop top N items if at least N items are present. The top item in
    /// the stack will be at the *end* of list of returned items.
    pub fn pop_n(&mut self, n: usize) -> Option<Vec<T>> {
        let size = self.size();
        if size < n {
            None
        } else {
            let items = self.storage.split_off(size - n);
            Some(items)
        }
    }

    pub fn peek(&self) -> Option<&T> {
        self.storage.last()
    }

    pub fn peek_at(&self, index: usize) -> Option<&T> {
        self.storage.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    pub fn size(&self) -> usize {
        self.storage.len()
    }

    #[cfg(test)]
    pub fn clear(&mut self) {
        self.storage.clear()
    }

    pub fn split_off(&mut self, len: usize) -> Vec<T> {
        self.storage.split_off(len)
    }

    pub fn truncate(&mut self, len: usize) {
        self.storage.truncate(len)
    }

    pub fn iter(&self) -> Rev<Iter<T>> {
        self.storage.iter().rev()
    }
}

impl fmt::Display for Stack<usize> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for index in self.iter() {
            write!(f, "{}", index)?;
        }
        write!(f, "")
    }
}
