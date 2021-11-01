//! The FeInt virtual machine. When it's created, it's initialized and
//! then, implicitly, goes idle until it's passed some instructions to
//! execute. After instructions are executed, it goes back into idle
//! mode.
use std::fmt;
use std::rc::Rc;

use crate::ast;
use crate::compiler::compile;
use crate::parser::{self, ParseResult};
use crate::util::{BinaryOperator, Stack, UnaryOperator};

use super::context::RuntimeContext;
use super::frame::Frame;
use super::inst::{Chunk, Inst};
use super::result::{ExeResult, RuntimeErr, RuntimeErrKind, VMState};
use crate::types::{ObjectRef, String, Tuple};

type RustString = std::string::String;

/// Execute source text.
pub fn execute_text(vm: &mut VM, text: &str, dis: bool, debug: bool) -> ExeResult {
    execute_parse_result(vm, parser::parse_text(text, debug), dis, debug)
}

/// Execute source from file.
pub fn execute_file(vm: &mut VM, path: &str, dis: bool, debug: bool) -> ExeResult {
    execute_parse_result(vm, parser::parse_file(path, debug), dis, debug)
}

/// Execute source from stdin.
pub fn execute_stdin(vm: &mut VM, dis: bool, debug: bool) -> ExeResult {
    execute_parse_result(vm, parser::parse_stdin(debug), dis, debug)
}

/// Execute parse result.
pub fn execute_parse_result(
    vm: &mut VM,
    result: ParseResult,
    dis: bool,
    debug: bool,
) -> ExeResult {
    match result {
        Ok(program) => execute_program(vm, program, dis, debug),
        Err(err) => Err(RuntimeErr::new(RuntimeErrKind::ParseError(err))),
    }
}

/// Create a new VM and execute AST program.
pub fn execute_program(
    vm: &mut VM,
    program: ast::Program,
    dis: bool,
    debug: bool,
) -> ExeResult {
    match compile(vm, program, debug) {
        Ok(instructions) => execute(vm, instructions, dis, debug),
        Err(err) => Err(RuntimeErr::new(RuntimeErrKind::CompilationError(err))),
    }
}

pub fn execute(vm: &mut VM, instructions: Chunk, dis: bool, debug: bool) -> ExeResult {
    let result = if cfg!(debug_assertions) {
        if dis {
            eprintln!("{:=<72}", "INSTRUCTIONS ");
        } else if debug {
            eprintln!("{:=<72}", "OUTPUT ");
        }
        vm.execute(instructions, dis)
    } else if dis {
        eprintln!("{:=<72}", "INSTRUCTIONS ");
        let result = vm.dis_list(&instructions);
        eprintln!("NOTE: Full disassembly is only available in debug builds");
        result
    } else {
        if debug {
            eprintln!("{:=<72}", "OUTPUT ");
        }
        vm.execute(instructions, false)
    };
    if debug {
        eprintln!("{:=<72}", "STACK ");
        vm.display_stack();
        eprintln!("{:=<72}", "VM STATE ");
        eprintln!("{:?}", result);
    }
    result
}

pub struct VM {
    pub ctx: RuntimeContext,
    stack: Stack<usize>,
    call_stack: Stack<Frame>,
}

impl Default for VM {
    fn default() -> Self {
        VM::new(RuntimeContext::default())
    }
}

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
    fn execute(&mut self, instructions: Chunk, dis: bool) -> ExeResult {
        let mut ip: usize = 0;

        loop {
            match &instructions[ip] {
                Inst::NoOp => {
                    // do nothing
                }
                Inst::Push(value) => {
                    self.stack.push(*value);
                }
                Inst::Pop => {
                    if self.stack.is_empty() {
                        self.err(RuntimeErrKind::EmptyStack)?;
                    }
                    self.stack.pop();
                }
                Inst::Jump(address) => {
                    #[cfg(debug_assertions)]
                    self.dis(dis, ip, &instructions);
                    ip = *address;
                    continue;
                }
                Inst::LoadConst(index) => {
                    self.format_strings(*index)?;
                    self.stack.push(*index);
                }
                Inst::DeclareVar(name) => {
                    // NOTE: Currently, declaration and assignment are
                    //       the same thing, so declaration doesn't
                    //       do anything particularly useful ATM.
                    self.ctx.declare_var(name.as_str());
                }
                Inst::AssignVar(name) => {
                    if let Some(i) = self.pop() {
                        self.ctx.assign_var(name, i);
                        self.stack.push(i);
                    } else {
                        let message = format!("Assignment");
                        self.err(RuntimeErrKind::NotEnoughValuesOnStack(message))?;
                    };
                }
                Inst::LoadVar(name) => {
                    if let Some(&index) = self.ctx.get_obj_index(name) {
                        self.format_strings(index)?;
                        self.stack.push(index);
                    } else {
                        self.err(RuntimeErrKind::NameError(format!(
                            "Name not found: {}",
                            name
                        )))?;
                    }
                }
                Inst::UnaryOp(op) => {
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
                                #[cfg(debug_assertions)]
                                self.dis(dis, ip, &instructions);
                                ip += 1;
                                continue;
                            }
                        };
                        let index = self.ctx.constants.add(value);
                        self.stack.push(index);
                    } else {
                        let message = format!("Unary op: {}", op);
                        self.err(RuntimeErrKind::NotEnoughValuesOnStack(message))?;
                    };
                }
                Inst::BinaryOp(op) => {
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
                                #[cfg(debug_assertions)]
                                self.dis(dis, ip, &instructions);
                                continue;
                            }
                        };
                        let index = self.ctx.constants.add(value);
                        self.stack.push(index);
                    } else {
                        let message = format!("Binary op: {}", op);
                        self.err(RuntimeErrKind::NotEnoughValuesOnStack(message))?;
                    };
                }
                Inst::ScopeStart => {
                    self.ctx.enter_scope();
                }
                Inst::ScopeEnd(count) => {
                    for _ in 0..*count {
                        self.ctx.exit_scope();
                    }
                }
                Inst::Print => match self.stack.peek() {
                    Some(index) => {
                        let val = self.ctx.constants.get(*index).unwrap();
                        let mut print;
                        if cfg!(debug_assertions) {
                            self.dis(dis, ip, &instructions);
                            print = !dis;
                        } else {
                            print = true;
                        }
                        if print {
                            if let Some(tuple) = val.as_any().downcast_ref::<Tuple>() {
                                let items: Vec<RustString> = tuple
                                    .items()
                                    .into_iter()
                                    .map(|i| format!("{}", i))
                                    .collect();
                                println!("{}", items.join(" "));
                            } else {
                                println!("{}", val);
                            }
                        }
                        self.stack.pop();
                    }
                    None => {
                        self.err(RuntimeErrKind::EmptyStack)?;
                    }
                },
                Inst::Return => {
                    // TODO: Implement actual return
                    match self.stack.pop() {
                        Some(v) => println!("{}", v),
                        None => eprintln!("Stack is empty!"),
                    }
                }
                Inst::Halt(code) => {
                    self.halt();
                    #[cfg(debug_assertions)]
                    self.dis(dis, ip, &instructions);
                    break Ok(VMState::Halted(*code));
                }
                inst => {
                    let message = format!("{:?}", inst);
                    self.err(RuntimeErrKind::UnhandledInstruction(message))?;
                }
            }

            #[cfg(debug_assertions)]
            if instructions[ip] != Inst::Print {
                self.dis(dis, ip, &instructions);
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

    pub fn pop(&mut self) -> Option<usize> {
        self.stack.pop()
    }

    pub fn peek(&self) -> Option<&usize> {
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

    fn err(&self, kind: RuntimeErrKind) -> ExeResult {
        Err(RuntimeErr::new(kind))
    }

    /// Format strings.
    ///
    /// This is called whenever an object is loaded (constant or var).
    /// If the object isn't a $ string or a tuple, this does nothing.
    ///
    /// If the object a $ string, it will be formatted and the formatted
    /// value will *replace* the original constant value.
    ///
    /// If the object a tuple, any $ string items will be formatted. If
    /// any $ strings are present, the original tuple will be *replaced*
    /// with a new tuple containing the formatted values.
    fn format_strings(&mut self, const_index: usize) -> Result<(), RuntimeErr> {
        if let Some(obj) = self.ctx.get_obj(const_index) {
            if let Some(string) = obj.as_any().downcast_ref::<String>() {
                if string.is_format_string() {
                    let formatted = string.format(self)?;
                    let formatted = Rc::new(formatted);
                    self.ctx.constants.replace(const_index, formatted);
                }
            }
            if let Some(tuple) = obj.as_any().downcast_ref::<Tuple>() {
                let mut new_items: Vec<ObjectRef> = Vec::new();
                let mut num_formatted = 0;
                for item in tuple.items() {
                    if let Some(string) = item.as_any().downcast_ref::<String>() {
                        if string.is_format_string() {
                            let formatted = string.format(self)?;
                            new_items.push(Rc::new(formatted));
                            num_formatted += 1;
                        } else {
                            new_items.push(item.clone());
                        }
                    } else {
                        new_items.push(item.clone());
                    }
                }
                if num_formatted > 0 {
                    let new_tuple = self.ctx.builtins.new_tuple(new_items);
                    self.ctx.constants.replace(const_index, new_tuple);
                }
            }
        }
        Ok(())
    }

    /// Show the contents of the stack (top first).
    pub fn display_stack(&self) {
        if self.stack.is_empty() {
            return eprintln!("[EMPTY]");
        }
        for (i, index) in self.stack.iter().enumerate() {
            let obj = self.ctx.get_obj(*index);
            match obj {
                Some(obj) => {
                    eprintln!("{:0>4} ({}) -> {:?}", i, index, obj)
                }
                None => eprintln!("{:0>4} ({}) -> [NOT AN OBJECT]", i, index),
            }
        }
    }

    // Disassembler ----------------------------------------------------
    //
    // This is done here because we need the VM context in order to show
    // more useful info like jump targets, values, etc.

    /// Disassemble a list of instructions.
    pub fn dis_list(&mut self, instructions: &Chunk) -> ExeResult {
        for (ip, _) in instructions.iter().enumerate() {
            self.dis(true, ip, instructions);
        }
        Ok(VMState::Halted(0))
    }

    /// Disassemble a single instruction. The `flag` arg is so that
    /// we don't have to wrap every call in `if dis { self.dis(...) }`.
    pub fn dis(&self, flag: bool, ip: usize, instructions: &Chunk) {
        if flag {
            let inst = &instructions[ip];
            let formatted = self.format_instruction(instructions, inst);
            eprintln!("{:0>4} {}", ip, formatted);
        }
    }

    fn format_instruction(&self, instructions: &Chunk, inst: &Inst) -> RustString {
        use Inst::*;
        match inst {
            NoOp => format!("NOOP"),
            Push(index) => {
                let obj = self.ctx.get_obj(*index).unwrap();
                self.format_aligned("PUSH", format!("{} ({:?})", index, obj))
            }
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
                        format!("{} -> {} ({:?})", address, index, obj),
                    )
                }
                None => {
                    self.format_aligned("JUMP IF", format!("{} -> [EMPTY]", address))
                }
            },
            LoadConst(index) => {
                let obj = self.ctx.get_obj(*index).unwrap();
                self.format_aligned("LOAD_CONST", format!("{} ({:?})", index, obj))
            }
            DeclareVar(name) => self.format_aligned("DECLARE_VAR", name),
            AssignVar(name) => {
                let index = self.peek().unwrap_or(&0);
                let obj = self.ctx.get_obj(*index).unwrap();
                self.format_aligned("ASSIGN_VAR", format!("{} ({:?})", name, obj))
            }
            LoadVar(name) => {
                let index = self.peek().unwrap_or(&0);
                let obj = self.ctx.get_obj(*index).unwrap();
                self.format_aligned("LOAD_VAR", format!("{} ({:?})", name, obj))
            }
            UnaryOp(operator) => self.format_aligned("UNARY_OP", operator),
            BinaryOp(operator) => self.format_aligned("BINARY_OP", operator),
            ScopeStart => format!("SCOPE_START"),
            ScopeEnd(count) => self.format_aligned("SCOPE_END", count),
            Print => match self.peek() {
                Some(index) => {
                    let obj = self.ctx.get_obj(*index).unwrap();
                    self.format_aligned("PRINT", format!("{} ({:?})", index, obj))
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
        let instructions: Chunk = vec![
            Inst::LoadConst(i),
            Inst::LoadConst(j),
            Inst::BinaryOp(BinaryOperator::Add),
            Inst::Print,
            Inst::Halt(0),
        ];
        if let Ok(result) = vm.execute(instructions, false) {
            assert_eq!(result, VMState::Halted(0));
        }
    }
}
