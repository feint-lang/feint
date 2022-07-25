//! The FeInt virtual machine. When it's created, it's initialized and
//! then, implicitly, goes idle until it's passed some instructions to
//! execute. After instructions are executed, it goes back into idle
//! mode.
use std::collections::HashMap;
use std::fmt;

use crate::types::ObjectRef;
use crate::util::{BinaryOperator, Stack, UnaryOperator};

use super::context::RuntimeContext;
use super::frame::Frame;
use super::inst::{Chunk, Inst};
use super::result::{ExeResult, RuntimeErr, RuntimeErrKind, VMState};

type ValueStackType = (Option<usize>, usize);

pub struct VM {
    pub ctx: RuntimeContext,

    // The value stack contains pointers to both constants and vars.
    // Each item is a tuple of an optional namespace depth and an object
    // index. If the namespace depth is set for an entry on the stack,
    // that indicates that it refers to a var and its value will be
    // looked up through its namespace. Otherwise, the value will be
    // retrieved from constant storage.
    value_stack: Stack<ValueStackType>,

    call_stack: Stack<Frame>,
}

impl Default for VM {
    fn default() -> Self {
        VM::new(RuntimeContext::default())
    }
}

impl VM {
    pub fn new(ctx: RuntimeContext) -> Self {
        VM { ctx, value_stack: Stack::new(), call_stack: Stack::new() }
    }

    /// Execute the specified instructions and return the VM's state. If
    /// a HALT instruction isn't encountered, the VM will go "idle"; it
    /// will maintain its internal state and await further instructions.
    /// When a HALT instruction is encountered, the VM's state will be
    /// cleared; it can be "restarted" by passing more instructions to
    /// execute.
    pub fn execute(&mut self, chunk: &Chunk, dis: bool) -> ExeResult {
        use Inst::*;
        use RuntimeErrKind::*;

        let mut ip: usize = 0;

        loop {
            match &chunk[ip] {
                NoOp => {
                    // do nothing
                }
                // Constants
                Push(value) => {
                    self.push(None, *value);
                }
                Pop => {
                    if self.value_stack.is_empty() {
                        self.err(EmptyStack)?;
                    }
                    self.pop();
                }
                LoadConst(index) => {
                    self.push(None, *index);
                }
                // Scopes
                ScopeStart => {
                    self.ctx.enter_scope();
                }
                ScopeEnd(count) => {
                    self.ctx.exit_scopes(count);
                }
                // Vars
                DeclareVar(name) => {
                    if self.ctx.get_var_in_current_namespace(name).is_err() {
                        self.ctx.declare_var(name.as_str())?;
                    }
                }
                AssignVar(name) => {
                    if let Some((const_depth, const_index)) = self.pop() {
                        let obj = self.get_value(const_depth, const_index).unwrap();
                        let (depth, index) = self.ctx.assign_var(name, obj.clone())?;
                        self.push(Some(depth), index);
                    } else {
                        self.err(NotEnoughValuesOnStack(format!("Assignment")))?;
                    };
                }
                LoadVar(name) => {
                    let (depth, index) = self.ctx.var_index(name.as_str())?;
                    self.push(Some(depth), index);
                }
                // Jumps
                Jump(addr, scope_exit_count) => {
                    self.ctx.exit_scopes(&scope_exit_count);
                    if !dis {
                        ip = *addr;
                        continue;
                    }
                }
                JumpIf(addr, scope_exit_count) => {
                    self.ctx.exit_scopes(&scope_exit_count);
                    if let Some((depth, index)) = self.pop() {
                        let obj = self.get_value(depth, index).unwrap();
                        if obj.as_bool(&self.ctx)? {
                            if !dis {
                                ip = *addr;
                                continue;
                            }
                        }
                    } else {
                        return self.err(EmptyStack);
                    };
                }
                JumpIfNot(addr, scope_exit_count) => {
                    self.ctx.exit_scopes(&scope_exit_count);
                    if let Some((depth, index)) = self.pop() {
                        let obj = self.get_value(depth, index).unwrap();
                        if !obj.as_bool(&self.ctx)? {
                            if !dis {
                                ip = *addr;
                                continue;
                            }
                        }
                    } else {
                        return self.err(EmptyStack);
                    };
                }
                JumpIfElse(if_addr, else_addr, scope_exit_count) => {
                    self.ctx.exit_scopes(&scope_exit_count);
                    let addr = if let Some((depth, index)) = self.pop() {
                        let obj = self.get_value(depth, index).unwrap();
                        if obj.as_bool(&self.ctx)? {
                            *if_addr
                        } else {
                            *else_addr
                        }
                    } else {
                        return self.err(EmptyStack);
                    };
                    if !dis {
                        ip = addr;
                        continue;
                    }
                }
                // Operations
                UnaryOp(op) => {
                    use UnaryOperator::*;
                    let a = if let Some((depth, index)) = self.pop() {
                        self.get_value(depth, index).unwrap()
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
                            let index = self.ctx.add_const(result);
                            self.push(None, index);
                        }
                        // Operators that return bool
                        _ => {
                            let result = match op {
                                AsBool => a.as_bool(&self.ctx)?,
                                Not => a.not(&self.ctx)?,
                                _ => unreachable!(),
                            };
                            self.push(None, if result { 1 } else { 2 });
                        }
                    };
                }
                BinaryOp(op) => {
                    use BinaryOperator::*;
                    let (depth, index, a, b) =
                        if let Some(((depth_a, index_a), (depth_b, index_b))) =
                            self.pop_top_two()
                        {
                            let a = self.get_value(depth_a, index_a).unwrap();
                            let b = self.get_value(depth_b, index_b).unwrap();
                            (depth_a, index_a, a, b.clone())
                        } else {
                            return self.err(NotEnoughValuesOnStack(format!(
                                "Binary op: {}",
                                op
                            )));
                        };
                    match op {
                        // In place update operators
                        AddEqual | SubEqual => {
                            assert!(depth.is_some()); // lhs must be a var
                            let result = match op {
                                AddEqual => a.add(&b, &self.ctx)?,
                                SubEqual => a.sub(&b, &self.ctx)?,
                                _ => unreachable!(),
                            };
                            self.ctx.assign_var_by_depth_and_index(
                                depth.unwrap(),
                                index,
                                result,
                            )?;
                            self.push(depth, index);
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
                            let index = self.ctx.add_const(result);
                            self.push(None, index);
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
                            self.push(None, if result { 1 } else { 2 });
                        }
                    }
                }
                // Object construction
                MakeString(n) => {
                    if let Some(indices) = self.pop_n(*n) {
                        let mut string = String::with_capacity(32);
                        for (depth, index) in indices {
                            let obj = self.get_value(depth, index).unwrap();
                            string.push_str(obj.to_string().as_str());
                        }
                        let string_obj = self.ctx.builtins.new_string(string);
                        let string_i = self.ctx.add_const(string_obj);
                        self.push(None, string_i);
                    } else {
                        return self.err(NotEnoughValuesOnStack(format!(
                            "Format string: {n}"
                        )));
                    }
                }
                MakeTuple(n) => {
                    if let Some(indices) = self.pop_n(*n) {
                        let mut items = vec![];
                        for (depth, index) in indices {
                            let obj = self.get_value(depth, index).unwrap();
                            items.push(obj.clone());
                        }
                        let tuple = self.ctx.builtins.new_tuple(items);
                        let tuple_i = self.ctx.add_const(tuple);
                        self.push(None, tuple_i);
                    } else {
                        return self.err(NotEnoughValuesOnStack(format!("Tuple: {n}")));
                    }
                }
                // Functions
                Call(n) => match self.value_stack.pop_n(*n + 1) {
                    Some(indices) => {
                        // Get callable
                        let (depth, index) = indices[0];
                        let obj = self.get_value(depth, index).unwrap();
                        // Collect args
                        let num_args = *n;
                        let mut args: Vec<ObjectRef> = vec![];
                        if num_args > 0 {
                            for i in 1..indices.len() {
                                let (depth, index) = indices[i];
                                let arg = self.get_value(depth, index).unwrap();
                                args.push(arg.clone())
                            }
                        }
                        // Call
                        let result = obj.call(args, &self.ctx)?;
                        let return_val = match result {
                            Some(return_val) => return_val,
                            None => self.ctx.builtins.nil_obj.clone(),
                        };
                        let index = self.ctx.add_const(return_val);
                        self.push(None, index)
                    }
                    None => {
                        return self
                            .err(NotEnoughValuesOnStack(format!("Call: {}", *n + 1)));
                    }
                },
                Return => {
                    // TODO: Implement actual return
                }
                // Placeholders
                Placeholder(addr, inst, message) => {
                    self.halt();
                    eprintln!(
                        "Placeholder at {addr} was not updated: {inst:?}\n{message}"
                    );
                    break Ok(VMState::Halted(255));
                }
                BreakPlaceholder(addr, _) => {
                    self.halt();
                    eprintln!("Break placeholder at {addr} was not updated");
                    break Ok(VMState::Halted(255));
                }
                ContinuePlaceholder(addr, _) => {
                    self.halt();
                    eprintln!("Continue placeholder at {addr} was not updated");
                    break Ok(VMState::Halted(255));
                }
                // VM control
                Halt(code) => {
                    self.halt();
                    #[cfg(debug_assertions)]
                    self.dis(dis, ip, &chunk);
                    break Ok(VMState::Halted(*code));
                }
            }

            #[cfg(debug_assertions)]
            self.dis(dis, ip, &chunk);

            ip += 1;

            if ip == chunk.len() {
                break Ok(VMState::Idle);
            }
        }
    }

    pub fn halt(&mut self) {
        // TODO: Not sure what this should do or if it's even needed
    }

    // Const stack -----------------------------------------------------

    fn push(&mut self, depth: Option<usize>, index: usize) {
        self.value_stack.push((depth, index));
    }

    fn pop(&mut self) -> Option<ValueStackType> {
        self.value_stack.pop()
    }

    fn peek(&self) -> Option<&ValueStackType> {
        self.value_stack.peek()
    }

    fn pop_n(&mut self, n: usize) -> Option<Vec<ValueStackType>> {
        self.value_stack.pop_n(n)
    }

    fn pop_top_two(&mut self) -> Option<(ValueStackType, ValueStackType)> {
        match self.pop_n(2) {
            Some(items) => Some((items[0], items[1])),
            None => None,
        }
    }

    fn get_value(
        &self,
        depth: Option<usize>,
        index: usize,
    ) -> Result<&ObjectRef, RuntimeErr> {
        match depth {
            Some(depth) => self.ctx.get_var(depth, index),
            None => self.ctx.get_const(index),
        }
    }

    /// Get value corresponding to top of stack. If the stack is empty
    /// `None` is returned.
    pub fn get_top_value(&self) -> Result<Option<&ObjectRef>, RuntimeErr> {
        if let Some((depth, index)) = self.peek() {
            let value = self.get_value(*depth, *index)?;
            Ok(Some(value))
        } else {
            Ok(None)
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
    pub fn display_stack(&mut self) {
        if self.value_stack.is_empty() {
            return eprintln!("[EMPTY]");
        }
        for (i, (depth, index)) in self.value_stack.iter().enumerate() {
            let obj = self.get_value(*depth, *index);
            match obj {
                Ok(obj) => {
                    eprintln!("{:0>4} ({}) -> {:?}", i, index, obj)
                }
                Err(_) => eprintln!("{:0>4} ({}) -> [NOT AN OBJECT]", i, index),
            }
        }
    }

    // Disassembler ----------------------------------------------------
    //
    // This is done here because we need the VM context in order to show
    // more useful info like jump targets, values, etc.

    /// Disassemble a list of instructions.
    pub fn dis_list(&mut self, chunk: &Chunk) -> ExeResult {
        for (ip, _) in chunk.iter().enumerate() {
            self.dis(true, ip, chunk);
        }
        Ok(VMState::Halted(0))
    }

    /// Disassemble a single instruction. The `flag` arg is so that
    /// we don't have to wrap every call in `if dis { self.dis(...) }`.
    pub fn dis(&mut self, flag: bool, ip: usize, chunk: &Chunk) {
        if flag {
            let inst = &chunk[ip];
            let formatted = self.format_instruction(chunk, inst);
            eprintln!("{:0>4} {}", ip, formatted);
        }
    }

    fn format_instruction(&mut self, chunk: &Chunk, inst: &Inst) -> String {
        use Inst::*;

        let obj_str = |depth, index| match self.get_value(depth, index) {
            Ok(obj) => format!("{obj:?}").replace("\n", "\\n").replace("\r", "\\r"),
            Err(_) => format!("[Object not found at {index}]"),
        };

        match inst {
            NoOp => format!("NOOP"),
            Push(index) => {
                let obj_str = obj_str(None, *index);
                self.format_aligned("PUSH", format!("{index} ({obj_str})"))
            }
            Pop => format!("POP"),
            LoadConst(index) => {
                let obj_str = obj_str(None, *index);
                self.format_aligned("LOAD_CONST", format!("{index} ({obj_str})"))
            }
            ScopeStart => format!("SCOPE_START"),
            ScopeEnd(count) => self.format_aligned("SCOPE_END", count),
            DeclareVar(name) => self.format_aligned("DECLARE_VAR", name),
            AssignVar(name) => {
                let (depth, index) = self.peek().unwrap_or(&(None, 0));
                let obj_str = obj_str(*depth, *index);
                self.format_aligned(
                    "ASSIGN_VAR",
                    format!("{name} = {obj_str} (from {depth:?}:{index})"),
                )
            }
            LoadVar(name) => {
                let (depth, index) = self.peek().unwrap_or(&(None, 0));
                let obj_str = obj_str(*depth, *index);
                self.format_aligned(
                    "LOAD_VAR",
                    format!("{name} = {obj_str} (from {depth:?}:{index})"),
                )
            }
            Jump(addr, _) => self.format_aligned("JUMP", format!("{addr}",)),
            JumpIf(addr, _) => self.format_aligned("JUMP_IF", format!("{addr}",)),
            JumpIfNot(addr, _) => {
                self.format_aligned("JUMP_IF_NOT", format!("{addr}",))
            }
            JumpIfElse(if_addr, else_addr, _) => match self.peek() {
                Some((depth, index)) => {
                    let obj_str = obj_str(None, *index);
                    self.format_aligned(
                        "JUMP_IF_ELSE",
                        format!("{obj_str} (from {depth:?}:{index}) ? {if_addr} : {else_addr}"),
                    )
                }
                None => self.format_aligned(
                    "JUMP_IF_ELSE",
                    format!("[EMPTY] ? {if_addr} : {else_addr}"),
                ),
            },
            UnaryOp(operator) => self.format_aligned("UNARY_OP", operator),
            BinaryOp(operator) => self.format_aligned("BINARY_OP", operator),
            MakeString(n) => self.format_aligned("MAKE_STRING", n),
            MakeTuple(n) => self.format_aligned("MAKE_TUPLE", n),
            Return => format!("RETURN"),
            Halt(code) => self.format_aligned("HALT", code),
            Placeholder(addr, inst, message) => {
                let formatted_inst = self.format_instruction(chunk, inst);
                self.format_aligned(
                    "PLACEHOLDER",
                    format!("{formatted_inst} @ {addr} ({message})"),
                )
            }
            BreakPlaceholder(addr, _) => {
                self.format_aligned("PLACEHOLDER", format!("BREAK @ {addr}"))
            }
            ContinuePlaceholder(addr, _) => {
                self.format_aligned("PLACEHOLDER", format!("CONTINUE @ {addr}"))
            }
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
    use super::*;

    #[test]
    fn execute_simple_program() {
        let mut vm = VM::default();
        let i = vm.ctx.add_const(vm.ctx.builtins.new_int(1));
        let j = vm.ctx.add_const(vm.ctx.builtins.new_int(2));
        let chunk: Chunk = vec![
            Inst::LoadConst(i),
            Inst::LoadConst(j),
            Inst::BinaryOp(BinaryOperator::Add),
            Inst::Halt(0),
        ];
        if let Ok(result) = vm.execute(&chunk, false) {
            assert_eq!(result, VMState::Halted(0));
        }
    }
}
