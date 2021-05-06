use crate::frame::Frame;
use crate::instructions::Instruction;
use crate::namespace::Namespace;
use crate::stack::Stack;

#[derive(Debug)]
pub enum VMState {
    Idle,
    Halted(i32, Option<String>),
}

pub struct VM<'a> {
    namespace: Namespace,

    // Items are pushed onto or popped from the stack as instructions
    // are executed in the instruction list.
    stack: Stack<usize>,

    // A new stack frame is pushed for each call
    call_stack: Stack<&'a Frame<'a>>,
}

/// The FeInt virtual machine. When it's created, it's initialized and
/// then, implicitly, goes idle until it's passed some instructions to
/// execute. After instructions are executed
impl<'a> VM<'a> {
    pub fn new(namespace: Namespace) -> Self {
        VM {
            namespace,
            stack: Stack::new(),
            call_stack: Stack::new(),
        }
    }

    pub fn halt(&mut self) {
        self.namespace.reset();
        self.stack.clear();
        self.call_stack.clear();
    }

    /// Execute the specified instructions and return the VM's state. If
    /// a HALT instruction isn't encountered, the VM will go "idle"; it
    /// will maintain its internal state and await further instructions.
    /// When a HALT instruction is encountered, the VM's state will be
    /// cleared; it can be "restarted" by passing more instructions to
    /// execute.
    pub fn execute(&mut self, instructions: &Vec<Instruction>) -> VMState {
        let mut instruction_pointer = 0;
        loop {
            match instructions.get(instruction_pointer) {
                Some(instruction) => {
                    instruction_pointer += 1;
                    match self.execute_instruction(instruction) {
                        state @ VMState::Halted(_, _) => break state,
                        _ => (),
                    }
                },
                // Go idle
                None => break VMState::Idle,
            }
        }
    }

    /// Run the specified instruction and return the VM's state.
    pub fn execute_instruction(&mut self, instruction: &Instruction) -> VMState {
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
    /// items. If it doesn't, the stack remains unmodified.
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
        self.call_stack.push(frame);
    }

    fn pop_frame(&mut self) -> Option<&Frame> {
        self.call_stack.pop()
    }
}
