use crate::frame::Frame;
use crate::opcodes::OpCode;
use crate::stack::Stack;

type ExitData = (i32, String);
type ExitResult = Result<String, (i32, String)>;

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

    pub fn run(
        &mut self,
        instructions: &Vec<OpCode<'a>>,
        mut program_counter: usize,
    ) -> ExitResult {
        if instructions.len() == 0 {
            return Err((1, "At least one instruction is required".to_string()));
        }
        loop {
            match instructions.get(program_counter) {
                Some(OpCode::Halt(code)) => {
                    return match code {
                        0 => Ok(message),
                        code => Err((*code, message))
                    }
                },
                Some(instruction) => {
                    let exit_data = self.step((*instruction).clone());
                    match exit_data {
                        Some((code, message)) => {
                            return match code {
                                0 => Ok(message),
                                code => Err((code, message))
                            }
                        },
                        None => (),
                    }
                },
                None => {
                    return Err((i32::MAX, "Reached end of instructions".to_string()))
                }
            }
            program_counter += 1;
        }
    }

    /// Run the specified instruction. If the instruction is HALT, the
    /// exit code is returned.
    fn execute_instruction(&mut self, instruction: OpCode) {
        match instruction {
            OpCode::Halt(code) => (),
            OpCode::Push(v) => self.stack.push(v),
            OpCode::Add => {
                match self.pop_top_two() {
                    Some((a, b)) => self.stack.push(a + b),
                    None => (),
                };
            },
            op => {
                #[cfg(debug_assertions)]
                println!("{:?}", op);
            },
            _ => (),
        }
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
            },
            _ => None
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
