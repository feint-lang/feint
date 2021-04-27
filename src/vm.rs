use crate::opcodes::OpCode;
use crate::stack::Stack;

pub struct VM<T> {
    program_counter: usize,
    stack: Stack<T>,
}

impl<T> VM<T> {
    pub fn new() -> VM<T> {
        VM {
            program_counter: 0,
            stack: Stack::new(),
        }
    }

    pub fn run(&self, instructions: Vec<T>) {
        loop {

        }
    }

    fn push_op(&mut self, op: T) {
        self.stack.push(op);
    }

    fn pop_op(&mut self) -> Option<T> {
        self.stack.pop()
    }
}
