use std::collections::HashMap;
use std::rc::Rc;

use num_bigint::BigInt;

use crate::ast;
use crate::compiler::compile;
use crate::parser::{self, ParseResult};
use crate::types::builtins::{Float, Int};
use crate::types::{Builtins, ObjectRef};
use crate::util::{BinaryOperator, Stack, UnaryOperator};

use super::arena::ObjectStore;
use super::frame::Frame;
use super::instruction::{Instruction, Instructions};
use super::namespace::Namespace;
use super::result::{
    ExecutionResult as Result, RuntimeError, RuntimeErrorKind, VMState,
};
use crate::vm::result::VMState::Running;

/// Execute source text.
pub fn execute_text(vm: &mut VM, text: &str, debug: bool) -> Result {
    execute_parse_result(vm, parser::parse_text(text), debug)
}

/// Execute source from file.
pub fn execute_file(vm: &mut VM, file_path: &str, debug: bool) -> Result {
    execute_parse_result(vm, parser::parse_file(file_path), debug)
}

/// Execute source from stdin.
pub fn execute_stdin(vm: &mut VM, debug: bool) -> Result {
    execute_parse_result(vm, parser::parse_stdin(), debug)
}

/// Execute parse result.
fn execute_parse_result(vm: &mut VM, result: ParseResult, debug: bool) -> Result {
    match result {
        Ok(program) => execute_program(vm, program, debug),
        Err(err) => Err(RuntimeError::new(RuntimeErrorKind::ParseError(err))),
    }
}

/// Create a new VM and execute AST program.
fn execute_program(vm: &mut VM, program: ast::Program, debug: bool) -> Result {
    let result = compile(vm, program, debug);
    match result {
        Ok(instructions) => vm.execute(instructions),
        Err(err) => Err(RuntimeError::new(RuntimeErrorKind::CompilationError(err))),
    }
}

pub struct RuntimeContext {
    pub builtins: Builtins,
    pub arena: ObjectStore,
}

impl RuntimeContext {
    pub fn new(builtins: Builtins, arena: ObjectStore) -> Self {
        Self { builtins, arena }
    }
}

impl Default for RuntimeContext {
    fn default() -> Self {
        let builtins = Builtins::new();
        let mut object_store = ObjectStore::new();
        object_store.add(builtins.nil_obj.clone()); // 0
        object_store.add(builtins.true_obj.clone()); // 1
        object_store.add(builtins.false_obj.clone()); // 2
        RuntimeContext::new(builtins, object_store)
    }
}

pub struct VM {
    pub ctx: RuntimeContext,

    namespace: Namespace,

    // Items are pushed onto or popped from the stack as instructions
    // are executed in the instruction list.
    stack: Stack<usize>,

    // A new stack frame is pushed for each call
    call_stack: Stack<Frame>,
}

impl Default for VM {
    fn default() -> Self {
        let ctx = RuntimeContext::default();
        let namespace = Namespace::new(None);
        VM::new(ctx, namespace)
    }
}

/// The FeInt virtual machine. When it's created, it's initialized and
/// then, implicitly, goes idle until it's passed some instructions to
/// execute. After instructions are executed
impl VM {
    pub fn new(ctx: RuntimeContext, namespace: Namespace) -> Self {
        VM { ctx, namespace, stack: Stack::new(), call_stack: Stack::new() }
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
    pub fn execute(&mut self, instructions: Instructions) -> Result {
        for instruction in instructions.iter() {
            let result = self.execute_instruction(instruction)?;
            if let VMState::Halted(_) = result {
                return Ok(result);
            }
        }
        Ok(VMState::Idle)
    }

    fn err(&self, kind: RuntimeErrorKind) -> Result {
        Err(RuntimeError::new(kind))
    }

    /// Run the specified instruction and return the VM's state.
    pub fn execute_instruction(&mut self, instruction: &Instruction) -> Result {
        match instruction {
            Instruction::Print => match self.stack.pop() {
                Some(index) => {
                    if index != 0 {
                        let value = self.ctx.arena.get(index).unwrap();
                        println!("{}", value);
                    }
                }
                None => {
                    return self.err(RuntimeErrorKind::EmptyStack);
                }
            },
            Instruction::Push(value) => {
                // TODO: Check if empty and return err if so
                self.stack.push(*value);
            }
            Instruction::Pop => {
                // TODO: Check if empty and return err if so
                if self.stack.is_empty() {
                    panic!("Stack is empty");
                }
                self.stack.pop();
            }
            Instruction::LoadConst(index) => {
                self.stack.push(*index);
            }
            // FIXME: Probably shouldn't be using const storage for this
            Instruction::LoadByName(name) => {
                let result = self.ctx.arena.get_index_for_name(name);
                match result {
                    Some(index) => self.stack.push(*index),
                    _ => {
                        return self.err(RuntimeErrorKind::NameError(format!(
                            "Name not found: {}",
                            name
                        )))
                    }
                }
            }
            Instruction::UnaryOp(op) => {
                if let Some(a) = self.pop() {
                    let value = match op {
                        // FIXME: Implement and call Object methods
                        UnaryOperator::Plus => a,
                        UnaryOperator::Negate => a,
                        UnaryOperator::Not => !a,
                    };
                    self.stack.push(value);
                } else {
                    let message = format!("Unary op: {}", op);
                    self.err(RuntimeErrorKind::NotEnoughValuesOnStack(message))?;
                };
            }
            Instruction::BinaryOp(op) if op == &BinaryOperator::Assign => {
                if let Some((i, j)) = self.pop_top_two() {
                    // Point name at value
                    self.ctx.arena.set_index_for_name(i, j);
                    // The return value of an assignment is the assigned
                    // value
                    self.stack.push(j);
                } else {
                    let message = format!("Assignment");
                    self.err(RuntimeErrorKind::NotEnoughValuesOnStack(message))?;
                };
            }
            Instruction::BinaryOp(op) => {
                if let Some((i, j)) = self.pop_top_two() {
                    let a = self.ctx.arena.get(i).unwrap();
                    let b = self.ctx.arena.get(j).unwrap();
                    let b = b.clone();
                    let value = match op {
                        BinaryOperator::Equality => {
                            let result = a.is_equal(b, &self.ctx)?;
                            self.stack.push(if result { 1 } else { 2 });
                            return Ok(VMState::Running);
                        }
                        BinaryOperator::Raise => a.raise(b, &self.ctx)?,
                        BinaryOperator::Multiply => a.mul(b, &self.ctx)?,
                        BinaryOperator::Divide => a.div(b, &self.ctx)?,
                        BinaryOperator::FloorDiv => a.floor_div(b, &self.ctx)?,
                        BinaryOperator::Modulo => a.modulo(b, &self.ctx)?,
                        BinaryOperator::Add => a.add(b, &self.ctx)?,
                        BinaryOperator::Subtract => a.sub(b, &self.ctx)?,
                        _ => {
                            return self.err(RuntimeErrorKind::UnhandledInstruction(
                                format!("BinaryOp: {}", op),
                            ));
                        }
                    };
                    let index = self.ctx.arena.add(value);
                    self.stack.push(index);
                } else {
                    let message = format!("Binary op: {}", op);
                    self.err(RuntimeErrorKind::NotEnoughValuesOnStack(message))?;
                };
            }
            Instruction::Return => {
                // TODO: Implement actual return
                match self.stack.pop() {
                    Some(v) => println!("{}", v),
                    None => eprintln!("Stack is empty!"),
                }
            }
            Instruction::Halt(code) => {
                self.halt();
                return Ok(VMState::Halted(*code));
            }
            instruction => {
                let message = format!("{:?}", instruction);
                self.err(RuntimeErrorKind::UnhandledInstruction(message))?;
            }
        }
        Ok(VMState::Running)
    }

    fn push(&mut self, item: usize) {
        self.stack.push(item);
    }

    fn pop(&mut self) -> Option<usize> {
        self.stack.pop()
    }

    /// Pop top two items from stack *if* the stack has at least two
    /// items. If it doesn't, the stack remains unmodified.
    ///
    /// NOTE: The second item down the stack will be *first* and the
    ///       first item will be *second* in the returned tuple. This
    ///       puts the items in "logical" order instead of having to
    ///       remember to swap them in them in the calling code.
    fn pop_top_two(&mut self) -> Option<(usize, usize)> {
        let stack = &mut self.stack;
        match (stack.pop(), stack.pop()) {
            (Some(top), Some(next)) => Some((next, top)),
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

    fn push_frame(&mut self, frame: Frame) {
        self.call_stack.push(frame);
    }

    fn pop_frame(&mut self) -> Option<Frame> {
        self.call_stack.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Builtins;

    #[test]
    fn execute_simple_program() {
        let builtins = Builtins::new();
        let mut vm = VM::default();
        let i = vm.ctx.arena.add(vm.ctx.builtins.new_int(1));
        let j = vm.ctx.arena.add(vm.ctx.builtins.new_int(2));
        let instructions: Instructions = vec![
            Instruction::LoadConst(i),
            Instruction::LoadConst(j),
            Instruction::BinaryOp(BinaryOperator::Add),
            Instruction::Print,
            Instruction::Halt(0),
        ];
        if let Ok(result) = vm.execute(instructions) {
            assert_eq!(result, VMState::Halted(0));
        }
    }
}
