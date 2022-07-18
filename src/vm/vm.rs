//! The FeInt virtual machine. When it's created, it's initialized and
//! then, implicitly, goes idle until it's passed some instructions to
//! execute. After instructions are executed, it goes back into idle
//! mode.
use std::fmt;

use crate::util::{BinaryOperator, Stack, UnaryOperator};

use super::context::RuntimeContext;
use super::frame::Frame;
use super::inst::{Chunk, Inst};
use super::result::{ExeResult, RuntimeErr, RuntimeErrKind, VMState};

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
    pub fn execute(&mut self, instructions: Chunk, dis: bool) -> ExeResult {
        use Inst::*;
        use RuntimeErrKind::*;

        let mut ip: usize = 0;

        loop {
            match &instructions[ip] {
                NoOp => {
                    // do nothing
                }
                Push(value) => {
                    self.push(*value);
                }
                Pop => {
                    if self.stack.is_empty() {
                        self.err(EmptyStack)?;
                    }
                    self.pop();
                }
                ScopeStart => {
                    self.ctx.enter_scope();
                }
                ScopeEnd(count) => {
                    for _ in 0..*count {
                        self.ctx.exit_scope();
                    }
                }
                Jump(addr) => {
                    #[cfg(debug_assertions)]
                    self.dis(dis, ip, &instructions);
                    ip = *addr;
                    continue;
                }
                JumpIf(addr) => {
                    if let Some(i) = self.pop() {
                        let obj = self.ctx.get_obj(i).unwrap();
                        if obj.as_bool(&self.ctx)? {
                            ip = *addr;
                            continue;
                        }
                    } else {
                        self.err(EmptyStack)?;
                    };
                }
                JumpIfNot(addr) => {
                    if let Some(i) = self.pop() {
                        let obj = self.ctx.get_obj(i).unwrap();
                        if !obj.as_bool(&self.ctx)? {
                            ip = *addr;
                            continue;
                        }
                    } else {
                        self.err(EmptyStack)?;
                    };
                }
                JumpIfElse(if_addr, else_addr) => {
                    if let Some(i) = self.pop() {
                        let obj = self.ctx.get_obj(i).unwrap();
                        if obj.as_bool(&self.ctx)? {
                            ip = *if_addr;
                        } else {
                            ip = *else_addr;
                        }
                    } else {
                        self.err(EmptyStack)?;
                    };
                    continue;
                }
                Break => unreachable!("break was not converted to jump"),
                LoadConst(index) => {
                    self.push(*index);
                }
                DeclareVar(name) => {
                    // NOTE: Currently, declaration and assignment are
                    //       the same thing, so declaration doesn't
                    //       do anything particularly useful ATM.
                    self.ctx.declare_var(name.as_str());
                }
                AssignVar(name) => {
                    if let Some(i) = self.pop() {
                        self.ctx.assign_var(name, i);
                        self.push(i);
                    } else {
                        let message = format!("Assignment");
                        self.err(NotEnoughValuesOnStack(message))?;
                    };
                }
                LoadVar(name) => {
                    if let Some(&index) = self.ctx.get_obj_index(name) {
                        self.push(index);
                    } else {
                        self.err(NameErr(format!(
                            "Name not defined in current scope: {}",
                            name
                        )))?;
                    }
                }
                UnaryOp(op) => {
                    use UnaryOperator::*;
                    let a = if let Some(i) = self.pop() {
                        self.ctx.get_obj(i).unwrap()
                    } else {
                        return self
                            .err(NotEnoughValuesOnStack(format!("Unary op: {}", op)));
                    };
                    match op {
                        Plus | Negate => {
                            let result = match op {
                                Plus => a.clone(), // no-op
                                Negate => a.negate(&self.ctx)?,
                                _ => unreachable!(),
                            };
                            let index = self.ctx.add_obj(result);
                            self.push(index);
                        }

                        // Operators that return bool
                        _ => {
                            let result = match op {
                                AsBool => a.as_bool(&self.ctx)?,
                                Not => a.not(&self.ctx)?,
                                _ => unreachable!(),
                            };
                            self.push(if result { 1 } else { 2 });
                        }
                    };
                }
                BinaryOp(op) => {
                    use BinaryOperator::*;
                    let (i, a, b) = if let Some((i, j)) = self.pop_top_two() {
                        let a = self.ctx.get_obj(i).unwrap();
                        let b = self.ctx.get_obj(j).unwrap();
                        (i, a, b.clone())
                    } else {
                        return self
                            .err(NotEnoughValuesOnStack(format!("Binary op: {}", op)));
                    };
                    match op {
                        // In place update operators
                        AddEqual | SubEqual => {
                            let result = match op {
                                AddEqual => a.add(&b, &self.ctx)?,
                                SubEqual => a.sub(&b, &self.ctx)?,
                                _ => unreachable!(),
                            };
                            self.ctx.replace_obj(i, result);
                            self.push(i);
                        }
                        // Math operators
                        Pow | Mul | Div | FloorDiv | Mod | Add | Sub => {
                            let result = match op {
                                Pow => a.pow(&b, &self.ctx)?,
                                Mul => a.mul(&b, &self.ctx)?,
                                Div => a.div(&b, &self.ctx)?,
                                FloorDiv => a.floor_div(&b, &self.ctx)?,
                                Mod => a.modulo(&b, &self.ctx)?,
                                Add => a.add(&b, &self.ctx)?,
                                Sub => a.sub(&b, &self.ctx)?,
                                _ => unreachable!(),
                            };
                            let index = self.ctx.add_obj(result);
                            self.push(index);
                        }
                        // Operators that return bool
                        _ => {
                            let result = match op {
                                IsEqual => a.is_equal(&b, &self.ctx)?,
                                Is => a.class().is(&b.class()) && a.id() == b.id(),
                                NotEqual => a.not_equal(&b, &self.ctx)?,
                                And => a.and(&b, &self.ctx)?,
                                Or => a.or(&b, &self.ctx)?,
                                LessThan => a.less_than(&b, &self.ctx)?,
                                LessThanOrEqual => {
                                    a.less_than(&b, &self.ctx)?
                                        || a.is_equal(&b, &self.ctx)?
                                }
                                GreaterThan => a.greater_than(&b, &self.ctx)?,
                                GreaterThanOrEqual => {
                                    a.greater_than(&b, &self.ctx)?
                                        || a.is_equal(&b, &self.ctx)?
                                }
                                _ => unreachable!(),
                            };
                            self.push(if result { 1 } else { 2 });
                        }
                    }
                }
                MakeString(n) => {
                    if let Some(indices) = self.pop_n(*n) {
                        let mut string = String::with_capacity(32);
                        for i in indices {
                            let obj = self.ctx.get_obj(i).unwrap();
                            string.push_str(obj.to_string().as_str());
                        }
                        let string_obj = self.ctx.builtins.new_string(string);
                        let string_i = self.ctx.add_obj(string_obj);
                        self.push(string_i);
                    } else {
                        return self.err(NotEnoughValuesOnStack(format!(
                            "Format string: {n}"
                        )));
                    }
                }
                MakeTuple(n) => {
                    if let Some(indices) = self.pop_n(*n) {
                        let mut items = vec![];
                        for i in indices {
                            let obj = self.ctx.get_obj(i).unwrap();
                            items.push(obj.clone());
                        }
                        let tuple = self.ctx.builtins.new_tuple(items);
                        let tuple_i = self.ctx.add_obj(tuple);
                        self.push(tuple_i);
                    } else {
                        return self.err(NotEnoughValuesOnStack(format!("Tuple: {n}")));
                    }
                }
                Print => match self.stack.peek() {
                    Some(index) => {
                        let val = self.ctx.get_obj(*index).unwrap();
                        let print;
                        if cfg!(debug_assertions) {
                            self.dis(dis, ip, &instructions);
                            print = !dis;
                        } else {
                            print = true;
                        }
                        if print {
                            println!("{}", val);
                        }
                        self.pop();
                    }
                    None => {
                        self.err(EmptyStack)?;
                    }
                },
                Return => {
                    // TODO: Implement actual return
                    match self.pop() {
                        Some(v) => println!("{}", v),
                        None => eprintln!("Stack is empty!"),
                    }
                }
                Halt(code) => {
                    self.halt();
                    #[cfg(debug_assertions)]
                    self.dis(dis, ip, &instructions);
                    break Ok(VMState::Halted(*code));
                }
                InternalErr(message) => {
                    self.halt();
                    eprintln!("INTERNAL ERROR: {}", message);
                    break Ok(VMState::Halted(255));
                }
            }

            #[cfg(debug_assertions)]
            if instructions[ip] != Print {
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

    fn pop(&mut self) -> Option<usize> {
        self.stack.pop()
    }

    pub fn peek(&self) -> Option<&usize> {
        self.stack.peek()
    }

    fn pop_n(&mut self, n: usize) -> Option<Vec<usize>> {
        self.stack.pop_n(n)
    }

    fn pop_top_two(&mut self) -> Option<(usize, usize)> {
        match self.pop_n(2) {
            Some(items) => Some((items[0], items[1])),
            None => None,
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

    fn format_instruction(&self, instructions: &Chunk, inst: &Inst) -> String {
        use Inst::*;
        match inst {
            NoOp => format!("NOOP"),
            Push(index) => {
                let obj = self.ctx.get_obj(*index).unwrap();
                self.format_aligned("PUSH", format!("{} ({:?})", index, obj))
            }
            Pop => format!("POP"),
            ScopeStart => format!("SCOPE_START"),
            ScopeEnd(count) => self.format_aligned("SCOPE_END", count),
            Jump(address) => self.format_aligned(
                "JUMP",
                format!(
                    "{} ({})",
                    address,
                    self.format_instruction(instructions, &instructions[*address])
                ),
            ),
            JumpIfElse(if_addr, else_addr) => match self.peek() {
                Some(index) => {
                    let obj = self.ctx.get_obj(*index).unwrap();
                    self.format_aligned(
                        "JUMP_IF_ELSE",
                        format!("{} ({}) ? {} : {:?}", obj, index, if_addr, else_addr),
                    )
                }
                None => self.format_aligned(
                    "JUMP_IF_ELSE",
                    format!("[EMPTY] ? {} : {}", if_addr, else_addr),
                ),
            },
            Break => unreachable!("break was not converted to jump"),
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
            MakeString(n) => self.format_aligned("MAKE_STRING", n),
            MakeTuple(n) => self.format_aligned("MAKE_TUPLE", n),
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
    fn format_aligned<T: fmt::Display>(&self, name: &str, value: T) -> String {
        format!("{: <w$}{: <x$}", name, value, w = 16, x = 4)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::Builtins;

    use super::*;

    #[test]
    fn execute_simple_program() {
        let mut vm = VM::default();
        let i = vm.ctx.add_obj(vm.ctx.builtins.new_int(1));
        let j = vm.ctx.add_obj(vm.ctx.builtins.new_int(2));
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
