use crate::stack::Stack;

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
