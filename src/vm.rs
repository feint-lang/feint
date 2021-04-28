use crate::frame::Frame;
use crate::instructions::Instruction;
use crate::stack::Stack;

pub enum VMState {
    Idle,
    Halted(i32, Option<String>),
}

pub struct VM<'a> {
    // Items are pushed onto or popped from the stack as op codes are
    // encountered in the instruction list.
    stack: Stack<usize>,

    // A new stack frame is pushed for each call
    frame_stack: Stack<&'a Frame<'a>>,
}

impl<'a> VM<'a> {
    pub fn new() -> VM<'a> {
        VM {
            stack: Stack::new(),
            frame_stack: Stack::new(),
        }
    }

    pub fn halt(&mut self) {
        self.stack.clear();
        self.frame_stack.clear();
    }

    /// Execute the specified instructions and return the VM's state.
    pub fn execute(&mut self, instructions: &Vec<Instruction>) -> VMState {
        let mut instruction_pointer = 0;

        loop {
            let instruction = instructions.get(instruction_pointer);
            instruction_pointer += 1;
            match instruction {
                Some(instruction) => match self.execute_instruction(instruction) {
                    VMState::Halted(code, message) => return VMState::Halted(code, message),
                    _ => (),
                },
                // Go idle
                None => break,
            }
        }

        VMState::Idle
    }

    /// Run the specified instruction and return the VM's state.
    fn execute_instruction(&mut self, instruction: &Instruction) -> VMState {
        match instruction {
            Instruction::Push(v) => {
                self.stack.push(*v);
            }
            Instruction::Add => {
                match self.pop_top_two() {
                    Some((a, b)) => self.stack.push(a + b),
                    None => (),
                };
            }
            Instruction::Print(n) => match self.stack.peek_n(*n) {
                items if items.len() == 0 => println!("Stack is empty"),
                items if items.len() == 1 => println!("{}", items.get(0).unwrap()),
                items => {
                    let mut iter = items.iter().rev();
                    let item = iter.next().unwrap();
                    println!("{} TOP", item);
                    while let Some(item) = iter.next() {
                        println!("{}", item);
                    }
                }
            },
            Instruction::Halt(code) => {
                self.halt();
                return VMState::Halted(*code, None);
            }
            instruction => {
                #[cfg(debug_assertions)]
                println!("{:?}", instruction);
            }
        }
        VMState::Idle
    }

    fn push(&mut self, item: usize) {
        self.stack.push(item);
    }

    fn pop(&mut self) -> Option<usize> {
        self.stack.pop()
    }

    /// Pop top two items from stack *if* the stack has at least two
    /// items.
    fn pop_top_two(&mut self) -> Option<(usize, usize)> {
        let stack = &mut self.stack;
        match (stack.pop(), stack.pop()) {
            (Some(top), Some(next)) => Some((top, next)),
            (Some(top), None) => {
                stack.push(top);
                None
            }
            _ => None,
        }
    }

    pub fn peek(&self) -> Option<&usize> {
        self.stack.peek()
    }

    fn push_frame(&mut self, frame: &'a Frame) {
        self.frame_stack.push(frame);
    }

    fn pop_frame(&mut self) -> Option<&Frame> {
        self.frame_stack.pop()
    }
}
