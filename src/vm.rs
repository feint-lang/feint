use crate::frame::Frame;
use crate::opcodes::OpCode;
use crate::stack::Stack;

type ExitResult = Result<String, (i32, String)>;

pub struct VM<'a> {
    // Items are pushed onto or popped from the stack as op codes are
    // encountered in the instruction list.
    instruction_stack: Stack<usize>,

    // A new stack frame is pushed for each call
    frame_stack: Stack<&'a Frame<'a>>,
}

impl<'a> VM<'a> {
    pub fn new() -> VM<'a> {
        VM {
            instruction_stack: Stack::new(),
            frame_stack: Stack::new(),
        }
    }

    pub fn run(
        &self,
        instructions: &Vec<OpCode>,
        mut program_counter: usize,
    ) -> ExitResult {
        if instructions.len() == 0 {
            return Err((1, "At least one instruction is required".to_string()));
        }

        loop {
            let instruction = instructions.get(program_counter);
            program_counter += 1;

            match instruction {
                Some(OpCode::Halt(0, message)) => {
                    return Ok(message.to_string());
                },
                Some(OpCode::Halt(exit_code, message)) => {
                    return Err((exit_code.clone(), message.to_string()));
                },
                Some(op) => {
                    #[cfg(debug_assertions)]
                    println!("{:?}", op);
                },
                None => {
                    // XXX: Should only happen in REPL. Otherwise, this
                    //      would only be reached when a list of
                    //      instructions doesn't include HALT at the
                    //      end.
                    return Err((i32::MAX, "Unexpected exit".to_string()));
                },
            }
        }
    }

    fn push(&mut self, item: usize) {
        self.instruction_stack.push(item);
    }

    fn pop(&mut self) -> Option<usize> {
        self.instruction_stack.pop()
    }

    fn push_frame(&mut self, frame: &'a Frame) {
        self.frame_stack.push(frame);
    }

    fn pop_frame(&mut self) -> Option<&Frame> {
        self.frame_stack.pop()
    }
}
