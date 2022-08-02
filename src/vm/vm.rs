//! The FeInt virtual machine. When it's created, it's initialized and
//! then, implicitly, goes idle until it's passed some instructions to
//! execute. After instructions are executed, it goes back into idle
//! mode.
use std::fmt;

use num_traits::ToPrimitive;

use crate::types::{create, Args, ObjectRef, ObjectTraitExt, Params};
use crate::util::{
    BinaryOperator, CompareOperator, InplaceOperator, Stack, UnaryCompareOperator,
    UnaryOperator,
};

use super::context::RuntimeContext;
use super::inst::{Chunk, Inst};
use super::result::{
    ExeResult, PeekObjResult, PopNObjResult, PopObjResult, RuntimeErr, RuntimeErrKind,
    RuntimeResult, VMState,
};

#[derive(Clone)]
pub enum ValueStackKind {
    Constant(usize),
    Var(usize, String),
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

    // Whenever an object is retrieved via dot notation, the object on
    // the left of the dot becomes the current bound method context
    // (AKA this).
    this: Option<ObjectRef>,
}

impl Default for VM {
    fn default() -> Self {
        VM::new(RuntimeContext::default())
    }
}

impl VM {
    pub fn new(ctx: RuntimeContext) -> Self {
        VM { ctx, value_stack: Stack::new(), scope_stack: Stack::new(), this: None }
    }

    pub fn with_argv(argv: Vec<&str>) -> Self {
        VM::new(RuntimeContext::with_argv(argv))
    }

    /// Execute the specified instructions and return the VM's state. If
    /// a HALT instruction isn't encountered, the VM will go "idle"; it
    /// will maintain its internal state and await further instructions.
    /// When a HALT instruction is encountered, the VM's state will be
    /// cleared; it can be "restarted" by passing more instructions to
    /// execute.
    pub fn execute(&mut self, chunk: &Chunk, dis: bool) -> ExeResult {
        use Inst::*;
        use ValueStackKind::*;

        let mut ip: usize = 0;
        let mut jump_ip = 0;
        let mut is_jump = false;

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
                    self.enter_scope();
                }
                ScopeEnd => {
                    self.exit_scopes(1);
                }
                // Vars
                DeclareVar(name) => {
                    if self.ctx.get_var_in_current_namespace(name).is_err() {
                        self.ctx.declare_var(name.as_str());
                    }
                }
                AssignVar(name) => {
                    let obj = self.pop_obj()?;
                    let depth = self.ctx.assign_var(name, obj)?;
                    self.push(Var(depth, name.clone()));
                }
                LoadVar(name) => {
                    let depth = self.ctx.get_var_depth(name.as_str())?;
                    self.push(Var(depth, name.clone()));
                }
                // Jumps
                Jump(addr, scope_exit_count) => {
                    self.exit_scopes(*scope_exit_count);
                    jump_ip = *addr;
                    is_jump = true;
                }
                JumpPushNil(addr, scope_exit_count) => {
                    self.push(Constant(0));
                    self.exit_scopes(*scope_exit_count);
                    jump_ip = *addr;
                    is_jump = true;
                }
                JumpIf(addr, scope_exit_count) => {
                    self.exit_scopes(*scope_exit_count);
                    let obj = self.pop_obj()?;
                    if obj.bool_val()? {
                        jump_ip = *addr;
                        is_jump = true;
                    }
                }
                JumpIfNot(addr, scope_exit_count) => {
                    self.exit_scopes(*scope_exit_count);
                    let obj = self.pop_obj()?;
                    if !obj.bool_val()? {
                        jump_ip = *addr;
                        is_jump = true;
                    }
                }
                JumpIfElse(if_addr, else_addr, scope_exit_count) => {
                    self.exit_scopes(*scope_exit_count);
                    let obj = self.pop_obj()?;
                    let addr = if obj.bool_val()? { *if_addr } else { *else_addr };
                    jump_ip = addr;
                    is_jump = true;
                }
                // Operations
                UnaryOp(op) => {
                    self.handle_unary_op(op)?;
                }
                UnaryCompareOp(op) => {
                    self.handle_unary_compare_op(op)?;
                }
                BinaryOp(op) => {
                    self.handle_binary_op(op)?;
                }
                CompareOp(op) => {
                    self.handle_compare_op(op)?;
                }
                InplaceOp(op) => {
                    self.handle_inplace_op(op)?;
                }
                // Object construction
                MakeString(n) => {
                    let objects = self.pop_n_obj(*n)?;
                    let mut string = String::with_capacity(32);
                    for obj in objects {
                        string.push_str(obj.to_string().as_str());
                    }
                    let string_obj = create::new_str(string);
                    self.push(Temp(string_obj));
                }
                MakeTuple(n) => {
                    let objects = self.pop_n_obj(*n)?;
                    let mut items = vec![];
                    for obj in objects {
                        items.push(obj.clone());
                    }
                    let tuple = create::new_tuple(items);
                    self.push(Temp(tuple));
                }
                // Functions
                Call(n) => {
                    self.handle_call(*n)?;
                }
                Return => {
                    // Return is a no-op
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
                    self.dis(dis, ip, chunk);
                    break Ok(VMState::Halted(*code));
                }
                HaltTop => {
                    let obj = self.pop_obj()?;
                    let return_code = match obj.get_int_val() {
                        Some(int) => {
                            self.halt();
                            #[cfg(debug_assertions)]
                            self.dis(dis, ip, chunk);
                            int.to_u8().unwrap_or(255)
                        }
                        None => 0,
                    };
                    break Ok(VMState::Halted(return_code));
                }
            }

            #[cfg(debug_assertions)]
            self.dis(dis, ip, chunk);

            if is_jump {
                ip = jump_ip;
                jump_ip = 0;
                is_jump = false;
            } else {
                ip += 1;
            }

            if ip == chunk.len() {
                break Ok(VMState::Idle);
            }
        }
    }

    // Handlers --------------------------------------------------------

    fn handle_unary_op(&mut self, op: &UnaryOperator) -> RuntimeResult {
        use UnaryOperator::*;
        let a = self.pop_obj()?;
        let result = match op {
            Plus => a, // no-op
            Negate => a.negate()?,
        };
        self.push(ValueStackKind::Temp(result));
        Ok(())
    }

    fn handle_unary_compare_op(&mut self, op: &UnaryCompareOperator) -> RuntimeResult {
        use UnaryCompareOperator::*;
        let a = self.pop_obj()?;
        let result = match op {
            AsBool => a.bool_val()?,
            Not => a.not()?,
        };
        let obj = create::bool_obj_from_bool(result);
        self.push(ValueStackKind::Temp(obj));
        Ok(())
    }

    fn get_binary_operands(&mut self) -> Result<(ObjectRef, ObjectRef), RuntimeErr> {
        let operands = self.pop_n_obj(2)?;
        let a = operands.get(0).unwrap().clone();
        let b = operands.get(1).unwrap().clone();
        Ok((a, b))
    }

    /// Pop top two operands from stack, apply operation, and push temp
    /// result value onto stack.
    fn handle_binary_op(&mut self, op: &BinaryOperator) -> RuntimeResult {
        use BinaryOperator::*;
        let (a, b) = self.get_binary_operands()?;
        let b = &*b;
        let result = match op {
            Pow => a.pow(b)?,
            Mul => a.mul(b)?,
            Div => a.div(b)?,
            FloorDiv => a.floor_div(b)?,
            Mod => a.modulo(b)?,
            Add => a.add(b)?,
            Sub => a.sub(b)?,
            Dot => {
                let obj = if let Some(name) = b.get_str_val() {
                    a.get_attr(name.as_str())?
                } else if let Some(index) = b.get_usize_val() {
                    a.get_item(index)?
                } else {
                    let message = format!("Not an attribute name or index: {b:?}");
                    return Err(RuntimeErr::new_type_err(message));
                };
                self.this = Some(a.clone());
                obj
            }
        };
        self.push(ValueStackKind::Temp(result));
        Ok(())
    }

    /// Pop top two operands from stack, compare them, and push bool
    /// temp value onto stack.
    fn handle_compare_op(&mut self, op: &CompareOperator) -> RuntimeResult {
        use CompareOperator::*;
        let (a, b) = self.get_binary_operands()?;
        let b = &*b;
        let result = match op {
            Is => a.is(b),
            IsEqual => a.is_equal(b),
            NotEqual => a.not_equal(b),
            And => a.and(b)?,
            Or => a.or(b)?,
            LessThan => a.less_than(b)?,
            LessThanOrEqual => a.less_than(b)? || a.is_equal(b),
            GreaterThan => a.greater_than(b)?,
            GreaterThanOrEqual => a.greater_than(b)? || a.is_equal(b),
        };
        let obj = create::bool_obj_from_bool(result);
        self.push(ValueStackKind::Temp(obj));
        Ok(())
    }

    /// Pop top two operands from stack, apply operation, assign result,
    /// and push temp result value onto stack. The first operand *must*
    /// be a variable.
    fn handle_inplace_op(&mut self, op: &InplaceOperator) -> RuntimeResult {
        use InplaceOperator::*;
        let (a_kind, a, b) = if let Some(kinds) = self.pop_n(2) {
            let a_kind = kinds[0].clone();
            let a = self.get_obj(kinds[0].clone())?;
            let b = self.get_obj(kinds[1].clone())?;
            (a_kind, a, b)
        } else {
            return Err(RuntimeErr::new(RuntimeErrKind::NotEnoughValuesOnStack(2)));
        };
        if let ValueStackKind::Var(depth, name) = a_kind {
            let b = &*b;
            let result = match op {
                AddEqual => a.add(b)?,
                SubEqual => a.sub(b)?,
            };
            self.ctx.assign_var_at_depth(depth, name.as_str(), result)?;
            self.push(ValueStackKind::Var(depth, name));
        } else {
            let message = format!("Binary op: {}", op);
            return Err(RuntimeErr::new(RuntimeErrKind::ExpectedVar(message)));
        }
        Ok(())
    }

    fn handle_call(&mut self, n: usize) -> RuntimeResult {
        use ValueStackKind::ReturnVal;
        let objects = self.pop_n_obj(n + 1)?;
        let callable = objects.get(0).unwrap();
        let mut args: Args = vec![];
        if objects.len() > 1 {
            for i in 1..objects.len() {
                args.push(objects.get(i).unwrap().clone());
            }
        }
        if let Some(func) = callable.down_to_builtin_func() {
            self.check_call_args(&func.name, &func.params, &args, false)?;
            let result = callable.call(self.this.clone(), args, self)?;
            if self.this.is_some() {
                self.this = None;
            }
            let return_val = match result {
                Some(return_val) => return_val,
                None => create::new_nil(),
            };
            // For builtin functions, the return value isn't on the
            // stack, so we have to put it there.
            self.push(ReturnVal(return_val));
        } else if let Some(func) = callable.down_to_func() {
            // Wrap the function call in a scope where the
            // function's locals are defined. After the
            // call, this scope will be cleared out.
            self.enter_scope();
            if let Some(this) = &self.this {
                self.ctx.declare_and_assign_var("this", this.clone())?;
                self.this = None;
            }
            self.check_call_args(&func.name, &func.params, &args, true)?;
            self.execute(&func.chunk, false)?;
            self.exit_scopes(1);
        } else {
            return Err(RuntimeErr::new_not_callable(callable.clone()));
        }
        Ok(())
    }

    /// Check call args to ensure they're valid. If they are, bind them
    /// to names in the call scope (if `bind` is specified).
    pub fn check_call_args(
        &mut self,
        name: &str,
        params: &Params,
        args: &Args,
        bind: bool,
    ) -> RuntimeResult {
        if let Some(params) = &params {
            let arity = params.len();
            let num_args = args.len();
            if num_args != arity {
                let ess = if arity == 1 { "" } else { "s" };
                return Err(RuntimeErr::new_type_err(format!(
                    "{name}() expected {arity} arg{ess}; got {num_args}"
                )));
            }
            if bind {
                for (name, arg) in params.iter().zip(args) {
                    self.ctx.declare_and_assign_var(name, arg.clone())?;
                }
            }
        } else if bind {
            let args = create::new_tuple(args.clone());
            self.ctx.declare_and_assign_var("$args", args)?;
        }
        Ok(())
    }

    pub fn enter_scope(&mut self) {
        self.scope_stack.push(self.value_stack.size());
        self.ctx.enter_scope();
    }

    /// When exiting a scope, we first save the top of the stack (which
    /// is the "return value" of the scope), remove all stack values
    /// added in the scope, and finally push the scope's "return value"
    /// back onto the stack. After taking care of the VM stack, the
    /// scope's namespace is then cleared and removed.
    fn exit_scopes(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        if count > 1 {
            let drop_count = count - 1;
            self.scope_stack.truncate(self.scope_stack.size() - drop_count);
            self.value_stack.truncate(self.value_stack.size() - drop_count);
        }
        let return_val = self.pop_obj();
        if let Some(size) = self.scope_stack.pop() {
            self.value_stack.truncate(size);
        } else {
            panic!("Scope stack unexpectedly empty when exiting scope(s): {count}");
        };
        if let Ok(obj) = return_val {
            self.push(ValueStackKind::Temp(obj));
        } else {
            panic!("Stack unexpectedly empty when exiting scope(s): {count}");
        }
        self.ctx.exit_scopes(count);
    }

    pub fn halt(&mut self) {
        // TODO: Not sure what this should do or if it's even needed
    }

    // Const stack -----------------------------------------------------

    pub(crate) fn push(&mut self, kind: ValueStackKind) {
        self.value_stack.push(kind);
    }

    fn pop(&mut self) -> Option<ValueStackKind> {
        self.value_stack.pop()
    }

    pub(crate) fn pop_obj(&mut self) -> PopObjResult {
        match self.pop() {
            Some(kind) => self.get_obj(kind),
            None => Err(RuntimeErr::new(RuntimeErrKind::EmptyStack)),
        }
    }

    fn pop_n(&mut self, n: usize) -> Option<Vec<ValueStackKind>> {
        self.value_stack.pop_n(n)
    }

    fn pop_n_obj(&mut self, n: usize) -> PopNObjResult {
        match self.pop_n(n) {
            Some(kinds) => {
                let mut objects = vec![];
                for kind in kinds {
                    objects.push(self.get_obj(kind)?);
                }
                Ok(objects)
            }
            None => Err(RuntimeErr::new(RuntimeErrKind::NotEnoughValuesOnStack(n))),
        }
    }

    fn peek(&self) -> Option<&ValueStackKind> {
        self.value_stack.peek()
    }

    pub fn peek_obj(&mut self) -> PeekObjResult {
        match self.peek() {
            Some(kind) => {
                let obj = self.get_obj(kind.clone())?;
                Ok(Some(obj))
            }
            None => Ok(None),
        }
    }

    fn get_obj(&self, kind: ValueStackKind) -> Result<ObjectRef, RuntimeErr> {
        use ValueStackKind::*;
        match kind {
            Constant(index) => Ok(self.ctx.get_const(index)?.clone()),
            Var(depth, name) => {
                let val = self.ctx.get_var_at_depth(depth, name.as_str())?;
                Ok(val.clone())
            }
            Temp(obj) => Ok(obj.clone()),
            ReturnVal(obj) => Ok(obj.clone()),
        }
    }

    // Utilities -------------------------------------------------------

    /// Show the contents of the stack (top first).
    pub fn display_stack(&self) {
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

    /// Show constants.
    pub fn display_constants(&self) {
        for (index, obj) in self.ctx.iter_constants().enumerate() {
            eprintln!("{index:0>8} {obj}");
        }
    }

    /// Show vars.
    pub fn display_vars(&self) {
        // TODO:
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

    /// Disassemble functions, returning the number of functions that
    /// were disassembled.
    pub fn dis_functions(&mut self) -> usize {
        let mut funcs = vec![];
        for obj in self.ctx.iter_constants() {
            let is_func = obj.down_to_func().is_some();
            if is_func {
                funcs.push(obj.clone());
            }
        }
        let num_funcs = funcs.len();
        for (i, func_obj) in funcs.iter().enumerate() {
            let func = func_obj.down_to_func().unwrap();
            let heading = format!("{func:?} ");
            eprintln!("{:=<79}", heading);
            if let Err(err) = self.dis_list(&func.chunk) {
                eprintln!("Could not disassemble function {func}: {err}");
            }
            if num_funcs > 1 && i < (num_funcs - 1) {
                eprintln!();
            }
        }
        num_funcs
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
                Ok(obj) => {
                    let class = obj.class();
                    let str = format!("{obj:?} <{class}>");
                    str.replace('\n', "\\n").replace('\r', "\\r")
                }
                Err(err) => format!("[ERROR: Could not get object: {err}]"),
            },
            None => "[Object not found]".to_string(),
        };

        match inst {
            NoOp => "NOOP".to_string(),
            Truncate(size) => self.format_aligned("TRUNCATE", format!("{size}")),
            LoadConst(index) => {
                let obj_str = obj_str(Some(&Constant(*index)));
                self.format_aligned("LOAD_CONST", format!("{index} : {obj_str}"))
            }
            ScopeStart => "SCOPE_START".to_string(),
            ScopeEnd => "SCOPE_END".to_string(),
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
            JumpPushNil(addr, _) => {
                self.format_aligned("JUMP_PUSH_NIL", format!("{addr}",))
            }
            JumpIf(addr, _) => self.format_aligned("JUMP_IF", format!("{addr}",)),
            JumpIfNot(addr, _) => {
                self.format_aligned("JUMP_IF_NOT", format!("{addr}",))
            }
            JumpIfElse(if_addr, else_addr, _) => {
                self.format_aligned("JUMP_IF_ELSE", format!("{if_addr} : {else_addr}"))
            }
            UnaryOp(operator) => self.format_aligned("UNARY_OP", operator),
            UnaryCompareOp(operator) => {
                self.format_aligned("UNARY_COMPARE_OP", operator)
            }
            BinaryOp(operator) => self.format_aligned("BINARY_OP", operator),
            CompareOp(operator) => self.format_aligned("COMPARE_OP", operator),
            InplaceOp(operator) => self.format_aligned("INPLACE_OP", operator),
            MakeString(n) => self.format_aligned("MAKE_STRING", n),
            MakeTuple(n) => self.format_aligned("MAKE_TUPLE", n),
            Call(n) => self.format_aligned("CALL", n),
            Return => "RETURN".to_string(),
            Halt(code) => self.format_aligned("HALT", code),
            HaltTop => {
                if let Ok(peek) = self.peek_obj() {
                    if let Some(code) = peek {
                        self.format_aligned("HALT_TOP", code.to_string())
                    } else {
                        self.format_aligned("HALT_TOP", "[ERROR: stack empty]")
                    }
                } else {
                    self.format_aligned(
                        "HALT_TOP",
                        "[ERROR: could not get top of stack]",
                    )
                }
            }
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
        }
    }

    /// Return a nicely formatted string of instructions.
    fn format_aligned<T: fmt::Display>(&self, name: &str, value: T) -> String {
        format!("{: <w$}{: <x$}", name, value, w = 16, x = 4)
    }
}
