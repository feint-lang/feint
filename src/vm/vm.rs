//! The FeInt virtual machine. When it's created, it's initialized and
//! then, implicitly, goes idle until it's passed some instructions to
//! execute. After instructions are executed, it goes back into idle
//! mode.
use std::fmt;

use num_traits::ToPrimitive;

use crate::types::ObjectRef;
use crate::util::{BinaryOperator, Stack, UnaryOperator};

use super::context::RuntimeContext;
use super::frame::Frame;
use super::inst::{Chunk, Inst};
use super::result::{ExeResult, RuntimeErr, RuntimeErrKind, VMState};

#[derive(Clone)]
enum ValueStackKind {
    Constant(usize),
    Var(usize, usize),
    Temp(ObjectRef),
    ReturnVal(ObjectRef),
}

pub struct VM {
    pub ctx: RuntimeContext,
    // The value stack contains "pointers" to the different value types:
    // constants, vars, temporaries, and return values.
    value_stack: Stack<ValueStackKind>,
    // The scope stack contains value stack sizes. Each size is the size
    // that the stack was just before a scope was entered. When a scope
    // is exited, these sizes are used to truncate the value stack back
    // to its previous size so that items can be freed.
    scope_stack: Stack<usize>,
    call_stack: Stack<Frame>,
}

impl Default for VM {
    fn default() -> Self {
        VM::new(RuntimeContext::default())
    }
}

impl VM {
    pub fn new(ctx: RuntimeContext) -> Self {
        VM {
            ctx,
            value_stack: Stack::new(),
            call_stack: Stack::new(),
            scope_stack: Stack::new(),
        }
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
        use ValueStackKind::*;

        let mut ip: usize = 0;

        loop {
            match &chunk[ip] {
                NoOp => {
                    // do nothing
                }
                Truncate(size) => {
                    self.value_stack.truncate(*size);
                }
                // Constants
                LoadConst(index) => {
                    self.push(Constant(*index));
                }
                // Scopes
                ScopeStart => {
                    self.scope_stack.push(self.value_stack.size());
                    self.ctx.enter_scope();
                }
                ScopeEnd => {
                    self.exit_scopes(1);
                }
                // Vars
                DeclareVar(name) => {
                    if self.ctx.get_var_in_current_namespace(name).is_err() {
                        self.ctx.declare_var(name.as_str())?;
                    }
                }
                AssignVar(name) => {
                    if let Some(obj) = self.pop_obj()? {
                        let (depth, index) = self.ctx.assign_var(name, obj)?;
                        self.push(Var(depth, index));
                    } else {
                        return self.err(NotEnoughValuesOnStack(format!("Assignment")));
                    }
                }
                LoadVar(name) => {
                    let (depth, index) = self.ctx.var_index(name.as_str())?;
                    self.push(Var(depth, index));
                }
                // Jumps
                Jump(addr, scope_exit_count) => {
                    self.exit_scopes(*scope_exit_count);
                    if !dis {
                        ip = *addr;
                        continue;
                    }
                }
                JumpIf(addr, scope_exit_count) => {
                    self.exit_scopes(*scope_exit_count);
                    if let Some(obj) = self.pop_obj()? {
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
                    self.exit_scopes(*scope_exit_count);
                    if let Some(obj) = self.pop_obj()? {
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
                    self.exit_scopes(*scope_exit_count);
                    let addr = if let Some(obj) = self.pop_obj()? {
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
                    let a = if let Some(obj) = self.pop_obj()? {
                        obj
                    } else {
                        return self
                            .err(NotEnoughValuesOnStack(format!("Unary op: {}", op)));
                    };
                    match op {
                        Plus | Negate => {
                            let result = match op {
                                Plus => a, // no-op
                                Negate => a.negate(&self.ctx)?,
                                _ => unreachable!(),
                            };
                            self.push(Temp(result));
                        }
                        // Operators that return bool
                        _ => {
                            let result = match op {
                                AsBool => a.as_bool(&self.ctx)?,
                                Not => a.not(&self.ctx)?,
                                _ => unreachable!(),
                            };
                            let obj = if result {
                                self.ctx.builtins.true_obj.clone()
                            } else {
                                self.ctx.builtins.false_obj.clone()
                            };
                            self.push(Temp(obj));
                        }
                    };
                }
                BinaryOp(op) => {
                    use BinaryOperator::*;
                    // Operands for the binary operation.
                    let (a_kind, a, b) = if let Some(kinds) = self.pop_n(2) {
                        let a_kind = kinds[0].clone();
                        let a = self.get_obj(kinds[0].clone())?;
                        let b = self.get_obj(kinds[1].clone())?;
                        (a_kind, a, b)
                    } else {
                        let message = format!("Binary op: {op}",);
                        return self.err(NotEnoughValuesOnStack(message));
                    };
                    match op {
                        Dot => {
                            let result = if let Some(name) = b.str_val() {
                                a.get_attribute(name.as_str(), &self.ctx)?
                            } else if let Some(int) = b.int_val() {
                                a.get_item(&int, &self.ctx)?
                            } else {
                                let message =
                                    format!("Not an attribute name or index: {b:?}");
                                return Err(RuntimeErr::new_type_err(message));
                            };
                            self.push(Temp(result));
                        }
                        // In-place update operators
                        AddEqual | SubEqual => {
                            if let Var(depth, index) = a_kind {
                                let result = match op {
                                    AddEqual => a.add(&b, &self.ctx)?,
                                    SubEqual => a.sub(&b, &self.ctx)?,
                                    _ => unreachable!(),
                                };
                                self.ctx.assign_var_by_depth_and_index(
                                    depth, index, result,
                                )?;
                                self.push(Var(depth, index));
                            } else {
                                return self
                                    .err(ExpectedVar(format!("Binary op: {}", op)));
                            }
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
                            self.push(Temp(result));
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
                            let obj = if result {
                                self.ctx.builtins.true_obj.clone()
                            } else {
                                self.ctx.builtins.false_obj.clone()
                            };
                            self.push(Temp(obj));
                        }
                    }
                }
                // Object construction
                MakeString(n) => {
                    if let Some(objects) = self.pop_n_obj(*n)? {
                        let mut string = String::with_capacity(32);
                        for obj in objects {
                            string.push_str(obj.to_string().as_str());
                        }
                        let string_obj = self.ctx.builtins.new_string(string);
                        let string_i = self.ctx.add_const(string_obj);
                        self.push(Constant(string_i));
                    } else {
                        return self.err(NotEnoughValuesOnStack(format!(
                            "Format string: {n}"
                        )));
                    }
                }
                MakeTuple(n) => {
                    if let Some(objects) = self.pop_n_obj(*n)? {
                        let mut items = vec![];
                        for obj in objects {
                            items.push(obj.clone());
                        }
                        let tuple = self.ctx.builtins.new_tuple(items);
                        let tuple_i = self.ctx.add_const(tuple);
                        self.push(Constant(tuple_i));
                    } else {
                        return self.err(NotEnoughValuesOnStack(format!("Tuple: {n}")));
                    }
                }
                // Functions
                Call(n) => match self.pop_n_obj(*n + 1)? {
                    Some(objects) => {
                        let callable = objects.get(0).unwrap();
                        let mut args: Vec<ObjectRef> = vec![];
                        if objects.len() > 1 {
                            for i in 1..objects.len() {
                                args.push(objects.get(i).unwrap().clone());
                            }
                        }
                        if let Some(native_func) = callable.as_native_func() {
                            if let Some(arity) = native_func.arity {
                                let num_args = args.len();
                                if num_args != arity as usize {
                                    let ess = if arity == 1 { "" } else { "s" };
                                    return Err(RuntimeErr::new_type_err(format!(
                                        "{}() expected {arity} arg{ess}; got {num_args}",
                                        native_func.name,
                                    )));
                                }
                            }
                            let result = callable.call(args, &self.ctx)?;
                            let return_val = match result {
                                Some(return_val) => return_val,
                                None => self.ctx.builtins.nil_obj.clone(),
                            };
                            self.push(ReturnVal(return_val));
                        } else if let Some(func) = callable.as_func() {
                            self.execute(&func.chunk, dis)?;
                        } else {
                            return self.err(NotCallable(callable.clone()));
                        };
                    }
                    None => {
                        return self
                            .err(NotEnoughValuesOnStack(format!("Call: {}", *n + 1)));
                    }
                },
                Return => {
                    // TODO:
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
                HaltTop => {
                    if let Some(obj) = self.pop_obj()? {
                        let return_code = match obj.int_val() {
                            Some(int) => {
                                self.halt();
                                #[cfg(debug_assertions)]
                                self.dis(dis, ip, &chunk);
                                int.to_u8().unwrap_or(255)
                            }
                            None => 0,
                        };
                        break Ok(VMState::Halted(return_code));
                    } else {
                        return self.err(EmptyStack);
                    }
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

    /// When exiting a scope, we first save the top of the stack (which
    /// is the "return value" of the scope), remove all stack values
    /// added in the scope, finally push the scope's "return value" back
    /// onto the stack. After taking care of the VM stack, the scope's
    /// namespace is then cleared and removed.
    fn exit_scopes(&mut self, count: usize) {
        for _ in 0..count {
            let top = self.pop();
            let size = self.scope_stack.pop().unwrap();
            self.value_stack.truncate(size);
            match top {
                Some(obj) => self.push(obj),
                None => (),
            }
        }
        self.ctx.exit_scopes(count);
    }

    pub fn halt(&mut self) {
        // TODO: Not sure what this should do or if it's even needed
    }

    // Const stack -----------------------------------------------------

    fn get_obj(&self, kind: ValueStackKind) -> Result<ObjectRef, RuntimeErr> {
        use ValueStackKind::*;
        match kind {
            Constant(index) => Ok(self.ctx.get_const(index)?.clone()),
            Var(depth, index) => {
                Ok(self.ctx.get_var_by_depth_and_index(depth, index)?.clone())
            }
            Temp(obj) => Ok(obj.clone()),
            ReturnVal(obj) => Ok(obj.clone()),
        }
    }

    fn push(&mut self, kind: ValueStackKind) {
        self.value_stack.push(kind);
    }

    fn pop(&mut self) -> Option<ValueStackKind> {
        self.value_stack.pop()
    }

    fn pop_obj(&mut self) -> Result<Option<ObjectRef>, RuntimeErr> {
        let obj = match self.pop() {
            Some(kind) => Some(self.get_obj(kind)?),
            None => None,
        };
        Ok(obj)
    }

    fn pop_n(&mut self, n: usize) -> Option<Vec<ValueStackKind>> {
        self.value_stack.pop_n(n)
    }

    fn pop_n_obj(&mut self, n: usize) -> Result<Option<Vec<ObjectRef>>, RuntimeErr> {
        match self.pop_n(n) {
            Some(kinds) => {
                let mut objects = vec![];
                for kind in kinds {
                    objects.push(self.get_obj(kind)?);
                }
                Ok(Some(objects))
            }
            None => Ok(None),
        }
    }

    fn peek(&self) -> Option<&ValueStackKind> {
        self.value_stack.peek()
    }

    pub fn peek_obj(&mut self) -> Result<Option<ObjectRef>, RuntimeErr> {
        let obj = match self.peek() {
            Some(kind) => Some(self.get_obj(kind.clone())?),
            None => None,
        };
        Ok(obj)
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
        for (i, kind) in self.value_stack.iter().enumerate() {
            let obj = self.get_obj(kind.clone());
            match obj {
                Ok(obj) => {
                    eprintln!("{:0>8} {:?}", i, obj)
                }
                Err(_) => eprintln!("{:0>4} [NOT AN OBJECT]", i),
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
        use ValueStackKind::*;

        let obj_str = |kind_opt: Option<&ValueStackKind>| match kind_opt {
            Some(kind) => match self.get_obj(kind.clone()) {
                Ok(obj) => format!("{obj:?}").replace("\n", "\\n").replace("\r", "\\r"),
                Err(err) => format!("[ERROR: Could not get object: {err}]"),
            },
            None => format!("[Object not found]"),
        };

        match inst {
            NoOp => format!("NOOP"),
            Truncate(size) => self.format_aligned("TRUNCATE", format!("{size}")),
            LoadConst(index) => {
                let obj_str = obj_str(Some(&Constant(*index)));
                self.format_aligned("LOAD_CONST", format!("{obj_str}"))
            }
            ScopeStart => format!("SCOPE_START"),
            ScopeEnd => format!("SCOPE_END"),
            DeclareVar(name) => self.format_aligned("DECLARE_VAR", name),
            AssignVar(name) => {
                let obj_str = obj_str(self.peek());
                self.format_aligned("ASSIGN_VAR", format!("{name} = {obj_str}"))
            }
            LoadVar(name) => {
                let obj_str = obj_str(self.peek());
                self.format_aligned("LOAD_VAR", format!("{name} = {obj_str}"))
            }
            Jump(addr, _) => self.format_aligned("JUMP", format!("{addr}",)),
            JumpIf(addr, _) => self.format_aligned("JUMP_IF", format!("{addr}",)),
            JumpIfNot(addr, _) => {
                self.format_aligned("JUMP_IF_NOT", format!("{addr}",))
            }
            JumpIfElse(if_addr, else_addr, _) => {
                let obj_str = obj_str(self.peek());
                self.format_aligned(
                    "JUMP_IF_ELSE",
                    format!("{obj_str} ? {if_addr} : {else_addr}"),
                )
            }
            UnaryOp(operator) => self.format_aligned("UNARY_OP", operator),
            BinaryOp(operator) => self.format_aligned("BINARY_OP", operator),
            MakeString(n) => self.format_aligned("MAKE_STRING", n),
            MakeTuple(n) => self.format_aligned("MAKE_TUPLE", n),
            Call(n) => self.format_aligned("CALL", n),
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
