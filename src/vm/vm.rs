use crate::util::Stack;

use super::{
    BinaryOperator, Constant, ConstantStore, ExecutionError, ExecutionErrorKind,
    ExecutionResult, Frame, Instruction, Instructions, Namespace, VMState,
};

pub struct VM<'a> {
    namespace: Namespace,

    // Items are pushed onto or popped from the stack as instructions
    // are executed in the instruction list.
    stack: Stack<usize>,

    // A new stack frame is pushed for each call
    call_stack: Stack<&'a Frame<'a>>,

    constants: ConstantStore,
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
            constants: ConstantStore::new(),
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
    pub fn execute(&mut self, instructions: &Instructions) -> ExecutionResult {
        let mut instruction_pointer = 0;
        loop {
            if let Some(instruction) = instructions.get(instruction_pointer) {
                instruction_pointer += 1;
                if let Some(result) = self.execute_instruction(instruction) {
                    return result;
                };
            } else {
                // No instructions left. Note that from the point of
                // view of the VM, this is not an error.
                break Ok(VMState::Idle);
            }
        }
    }

    /// Run the specified instruction and return the VM's state.
    pub fn execute_instruction(
        &mut self,
        instruction: &Instruction,
    ) -> Option<ExecutionResult> {
        match instruction {
            Instruction::Pop => {
                // TODO: Check if empty and return err if so
                self.stack.pop().unwrap();
            }
            Instruction::StoreConst(value) => {
                self.constants.add(Constant::new(*value));
            }
            Instruction::LoadConst(index) => {
                let constant = self.constants.get(*index).unwrap();
                self.stack.push(constant.value); // ???
            }
            Instruction::BinaryOperation(operator) => {
                if let Some((a, b)) = self.pop_top_two() {
                    let value = match operator {
                        BinaryOperator::Multiply => a * b,
                        BinaryOperator::Divide => a / b,
                        BinaryOperator::Add => a + b,
                        BinaryOperator::Subtract => a - b,
                    };
                    self.stack.push(value);
                } else {
                    return Some(Err(ExecutionError::new(
                        ExecutionErrorKind::GenericError(
                            "Not enough values on stack".to_owned(),
                        ),
                    )));
                };
            }
            Instruction::Return => {
                // TODO: Implement actual return
                println!("{}", self.stack.pop().unwrap());
            }
            Instruction::Halt(code) => {
                self.halt();
                return Some(Ok(VMState::Halted(*code)));
            }
            instruction => {
                #[cfg(debug_assertions)]
                println!("{:?}", instruction);
            }
        }
        None
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

mod tests {
    use super::*;

    #[test]
    fn execute_simple_program() {
        let mut vm = VM::new(Namespace::new(None));
        let instructions: Instructions = vec![
            Instruction::StoreConst(1),
            Instruction::StoreConst(2),
            Instruction::LoadConst(0),
            Instruction::LoadConst(1),
            Instruction::BinaryOperation(BinaryOperator::Add),
            Instruction::Return,
            Instruction::Halt(0),
        ];
        if let Ok(result) = vm.execute(&instructions) {
            assert_eq!(result, VMState::Halted(0));
        }
    }
}
