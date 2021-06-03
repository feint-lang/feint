use std::fmt;
use std::rc::Rc;

use crate::ast;
use crate::compiler::compile;
use crate::parser::{self, ParseResult};
use crate::util::{BinaryOperator, Stack, UnaryOperator};

use super::context::RuntimeContext;
use super::frame::Frame;
use super::instruction::{Instruction, Instructions};
use super::result::{ExecutionResult, RuntimeError, RuntimeErrorKind, VMState};
use crate::types::String;

type RustString = std::string::String;

/// Execute source text.
pub fn execute_text(
    vm: &mut VM,
    text: &str,
    disassemble: bool,
    debug: bool,
) -> ExecutionResult {
    execute_parse_result(vm, parser::parse_text(text, debug), disassemble, debug)
}

/// Execute source from file.
pub fn execute_file(
    vm: &mut VM,
    file_path: &str,
    disassemble: bool,
    debug: bool,
) -> ExecutionResult {
    execute_parse_result(vm, parser::parse_file(file_path, debug), disassemble, debug)
}

/// Execute source from stdin.
pub fn execute_stdin(vm: &mut VM, disassemble: bool, debug: bool) -> ExecutionResult {
    execute_parse_result(vm, parser::parse_stdin(debug), disassemble, debug)
}

/// Execute parse result.
pub fn execute_parse_result(
    vm: &mut VM,
    result: ParseResult,
    disassemble: bool,
    debug: bool,
) -> ExecutionResult {
    match result {
        Ok(program) => execute_program(vm, program, disassemble, debug),
        Err(err) => Err(RuntimeError::new(RuntimeErrorKind::ParseError(err))),
    }
}

/// Create a new VM and execute AST program.
pub fn execute_program(
    vm: &mut VM,
    program: ast::Program,
    disassemble: bool,
    debug: bool,
) -> ExecutionResult {
    match compile(vm, program, debug) {
        Ok(instructions) => execute_instructions(vm, instructions, disassemble, debug),
        Err(err) => Err(RuntimeError::new(RuntimeErrorKind::CompilationError(err))),
    }
}

pub fn execute_instructions(
    vm: &mut VM,
    instructions: Instructions,
    disassemble: bool,
    debug: bool,
) -> ExecutionResult {
    match disassemble {
        true => vm.disassemble(instructions),
        false => {
            let result = vm.execute(instructions);
            if debug {
                for index in vm.stack.iter() {
                    let obj = vm.ctx.get_obj(*index);
                    match obj {
                        Some(obj) => {
                            eprintln!("{} {}({})", index, obj.class().name(), obj)
                        }
                        None => eprintln!("{} [NOT AN OBJECT]", index),
                    }
                }
            }
            result
        }
    }
}

pub struct VM {
    pub ctx: RuntimeContext,

    // Items are pushed onto or popped from the stack as instructions
    // are executed in the instruction list.
    stack: Stack<usize>,

    // A new stack frame is pushed for each call
    call_stack: Stack<Frame>,
}

impl Default for VM {
    fn default() -> Self {
        VM::new(RuntimeContext::default())
    }
}

/// The FeInt virtual machine. When it's created, it's initialized and
/// then, implicitly, goes idle until it's passed some instructions to
/// execute. After instructions are executed
impl VM {
    pub fn new(ctx: RuntimeContext) -> Self {
        VM { ctx, stack: Stack::new(), call_stack: Stack::new() }
    }

    pub fn halt(&mut self) {
        // TODO: Not sure what this should do or if it's even needed
    }

    /// Execute the specified instructions and return the VM's state. If
    /// a HALT instruction isn't encountered, the VM will go "idle"; it
    /// will maintain its internal state and await further instructions.
    /// When a HALT instruction is encountered, the VM's state will be
    /// cleared; it can be "restarted" by passing more instructions to
    /// execute.
    fn execute(&mut self, instructions: Instructions) -> ExecutionResult {
        let mut ip: usize = 0;

        loop {
            match &instructions[ip] {
                Instruction::NoOp => {
                    // do nothing
                }
                Instruction::Push(value) => {
                    self.stack.push(*value);
                }
                Instruction::Pop => {
                    if self.stack.is_empty() {
                        self.err(RuntimeErrorKind::EmptyStack)?;
                    }
                    self.stack.pop();
                }
                Instruction::Jump(address) => {
                    ip = *address;
                    continue;
                }
                Instruction::LoadConst(index) => {
                    self.format_string(*index)?;
                    self.stack.push(*index);
                }
                Instruction::DeclareVar(name) => {
                    // NOTE: Currently, declaration and assignment are
                    //       the same thing, so declaration doesn't
                    //       do anything particularly useful ATM.
                    self.ctx.declare_var(name.as_str());
                }
                Instruction::AssignVar(name) => {
                    if let Some(i) = self.pop() {
                        self.ctx.assign_var(name, i);
                        self.stack.push(i);
                    } else {
                        let message = format!("Assignment");
                        self.err(RuntimeErrorKind::NotEnoughValuesOnStack(message))?;
                    };
                }
                Instruction::LoadVar(name) => {
                    if let Some(&index) = self.ctx.get_obj_index(name) {
                        self.format_string(index)?;
                        self.stack.push(index);
                    } else {
                        self.err(RuntimeErrorKind::NameError(format!(
                            "Name not found: {}",
                            name
                        )))?;
                    }
                }
                Instruction::UnaryOp(op) => {
                    if let Some(i) = self.pop() {
                        let a = self.ctx.constants.get(i).unwrap();
                        let value = match op {
                            UnaryOperator::Plus => a.clone(), // no-op
                            UnaryOperator::Negate => a.negate(&self.ctx)?,
                            op => {
                                // Operators that return bool
                                let result = match op {
                                    UnaryOperator::AsBool => a.as_bool(&self.ctx)?,
                                    UnaryOperator::Not => a.not(&self.ctx)?,
                                    _ => unreachable!(),
                                };
                                self.stack.push(if result { 1 } else { 2 });
                                ip += 1;
                                continue;
                            }
                        };
                        let index = self.ctx.constants.add(value);
                        self.stack.push(index);
                    } else {
                        let message = format!("Unary op: {}", op);
                        self.err(RuntimeErrorKind::NotEnoughValuesOnStack(message))?;
                    };
                }
                Instruction::BinaryOp(op) => {
                    if let Some((i, j)) = self.pop_top_two() {
                        let a = self.ctx.constants.get(i).unwrap();
                        let b = self.ctx.constants.get(j).unwrap();
                        let b = b.clone();
                        let value = match op {
                            BinaryOperator::Pow => a.pow(b, &self.ctx)?,
                            BinaryOperator::Mul => a.mul(b, &self.ctx)?,
                            BinaryOperator::Div => a.div(b, &self.ctx)?,
                            BinaryOperator::FloorDiv => a.floor_div(b, &self.ctx)?,
                            BinaryOperator::Mod => a.modulo(b, &self.ctx)?,
                            BinaryOperator::Add => a.add(b, &self.ctx)?,
                            BinaryOperator::Sub => a.sub(b, &self.ctx)?,
                            op => {
                                // Operators that return bool
                                let result = match op {
                                    BinaryOperator::IsEqual => {
                                        a.is_equal(b, &self.ctx)?
                                    }
                                    BinaryOperator::NotEqual => {
                                        a.not_equal(b, &self.ctx)?
                                    }
                                    BinaryOperator::And => a.and(b, &self.ctx)?,
                                    BinaryOperator::Or => a.or(b, &self.ctx)?,
                                    _ => unreachable!(),
                                };
                                self.stack.push(if result { 1 } else { 2 });
                                ip += 1;
                                continue;
                            }
                        };
                        let index = self.ctx.constants.add(value);
                        self.stack.push(index);
                    } else {
                        let message = format!("Binary op: {}", op);
                        self.err(RuntimeErrorKind::NotEnoughValuesOnStack(message))?;
                    };
                }
                Instruction::BlockStart => {
                    self.ctx.enter_scope();
                }
                Instruction::BlockEnd(count) => {
                    for _ in 0..*count {
                        self.ctx.exit_scope();
                    }
                }
                Instruction::Print => match self.stack.pop() {
                    Some(index) => {
                        let value = self.ctx.constants.get(index).unwrap();
                        println!("{}", value);
                    }
                    None => {
                        self.err(RuntimeErrorKind::EmptyStack)?;
                    }
                },
                Instruction::Return => {
                    // TODO: Implement actual return
                    match self.stack.pop() {
                        Some(v) => println!("{}", v),
                        None => eprintln!("Stack is empty!"),
                    }
                }
                Instruction::Halt(code) => {
                    self.halt();
                    break Ok(VMState::Halted(*code));
                }
                instruction => {
                    let message = format!("{:?}", instruction);
                    self.err(RuntimeErrorKind::UnhandledInstruction(message))?;
                }
            }

            ip += 1;

            if ip == instructions.len() {
                break Ok(VMState::Idle);
            }
        }
    }

    // Stack -----------------------------------------------------------

    fn push(&mut self, item: usize) {
        self.stack.push(item);
    }

    fn pop(&mut self) -> Option<usize> {
        self.stack.pop()
    }

    fn peek(&self) -> Option<&usize> {
        self.stack.peek()
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

    // Call stack ------------------------------------------------------

    fn push_frame(&mut self, frame: Frame) {
        self.call_stack.push(frame);
    }

    fn pop_frame(&mut self) -> Option<Frame> {
        self.call_stack.pop()
    }

    // Utilities -------------------------------------------------------

    fn err(&self, kind: RuntimeErrorKind) -> ExecutionResult {
        Err(RuntimeError::new(kind))
    }

    /// Format String with vars from current context.
    fn format_string(&mut self, const_index: usize) -> Result<(), RuntimeError> {
        if let Some(obj) = self.ctx.get_obj(const_index) {
            if let Some(string) = obj.as_any().downcast_ref::<String>() {
                if string.is_format_string() {
                    let formatted = string.format(&self.ctx)?;
                    let formatted = Rc::new(formatted);
                    self.ctx.constants.replace(const_index, formatted);
                }
            }
        }
        Ok(())
    }

    // Disassembler ----------------------------------------------------
    //
    // This is done here because we need the VM context in order to show
    // more useful info like jump targets, values, etc.

    fn disassemble(&mut self, instructions: Instructions) -> ExecutionResult {
        let mut offset = 0;
        for instruction in instructions.iter() {
            eprintln!(
                "{:0>offset_width$} {}",
                offset,
                self.format_instruction(&instructions, instruction),
                offset_width = 4
            );
            offset += 1;
        }
        Ok(VMState::Halted(0))
    }

    fn format_instruction(
        &self,
        instructions: &Instructions,
        instruction: &Instruction,
    ) -> RustString {
        use Instruction::*;
        match instruction {
            NoOp => format!("NOOP"),
            Push(address) => self.format_aligned("PUSH", address),
            Pop => format!("POP"),
            Jump(address) => self.format_aligned(
                "JUMP",
                format!(
                    "{} ({})",
                    address,
                    self.format_instruction(instructions, &instructions[*address])
                ),
            ),
            JumpIfTrue(address) => match self.peek() {
                Some(index) => {
                    let obj = self.ctx.get_obj(*index).unwrap();
                    self.format_aligned(
                        "JUMP IF",
                        format!("{} -> {} ({})", address, index, obj),
                    )
                }
                None => {
                    self.format_aligned("JUMP IF", format!("{} -> [EMPTY]", address))
                }
            },
            LoadConst(index) => {
                let obj = self.ctx.get_obj(*index).unwrap();
                self.format_aligned("LOAD_CONST", format!("{} ({})", index, obj))
            }
            UnaryOp(operator) => self.format_aligned("UNARY_OP", operator),
            BinaryOp(operator) => self.format_aligned("BINARY_OP", operator),
            BlockStart => format!("BLOCK START"),
            BlockEnd(count) => self.format_aligned("BLOCK END", count),
            Print => match self.peek() {
                Some(index) => {
                    let obj = self.ctx.get_obj(*index).unwrap();
                    self.format_aligned("PRINT", format!("{} ({})", index, obj))
                }
                None => self.format_aligned("PRINT", "[EMPTY]"),
            },
            Return => format!("RETURN"),
            Halt(code) => self.format_aligned("HALT", code),
            other => format!("{:?}", other),
        }
    }

    /// Return a nicely formatted string of instructions.
    fn format_aligned<T: fmt::Display>(&self, name: &str, value: T) -> RustString {
        format!("{: <w$}{: <x$}", name, value, w = 16, x = 4)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::Builtins;

    use super::*;

    #[test]
    fn execute_simple_program() {
        let builtins = Builtins::new();
        let mut vm = VM::default();
        let i = vm.ctx.constants.add(vm.ctx.builtins.new_int(1));
        let j = vm.ctx.constants.add(vm.ctx.builtins.new_int(2));
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
