//! The FeInt virtual machine. When it's created, it's initialized and
//! then, implicitly, goes idle until it's passed some instructions to
//! execute. After instructions are executed, it goes back into idle
//! mode.
use num_traits::ToPrimitive;

use crate::types::{
    create, Args, BuiltinFunc, Func, ObjectRef, ObjectTraitExt, Params, This,
};
use crate::util::{
    BinaryOperator, CompareOperator, InplaceOperator, Stack, UnaryCompareOperator,
    UnaryOperator,
};
use crate::vm::result::PeekResult;

use super::code::Code;
use super::context::RuntimeContext;
use super::inst::Inst;
use super::result::{
    CallDepth, ExeResult, PeekObjResult, PopNObjResult, PopNResult, PopObjResult,
    PopResult, RuntimeErr, RuntimeErrKind, RuntimeResult, VMState, ValueStackKind,
};

pub const DEFAULT_MAX_CALL_DEPTH: CallDepth =
    if cfg!(debug_assertions) { 256 } else { 1024 };

struct CallFrame {
    _stack_position: usize,
}

impl CallFrame {
    pub fn new(_stack_position: usize) -> Self {
        Self { _stack_position }
    }
}

pub struct VM {
    pub ctx: RuntimeContext,
    // The value stack contains "pointers" to the different value types:
    // constants, vars, temporaries, and return values.
    value_stack: Stack<ValueStackKind>,
    // Pointer to stack position just before call.
    call_stack: Stack<CallFrame>,
    // Maximum depth of "call stack" (quotes because there's no explicit
    // call stack)
    max_call_depth: CallDepth,
}

impl Default for VM {
    fn default() -> Self {
        VM::new(RuntimeContext::new(), DEFAULT_MAX_CALL_DEPTH)
    }
}

impl VM {
    pub fn new(ctx: RuntimeContext, max_call_depth: CallDepth) -> Self {
        VM { ctx, value_stack: Stack::new(), call_stack: Stack::new(), max_call_depth }
    }

    /// Execute the specified instructions and return the VM's state. If
    /// a HALT instruction isn't encountered, the VM will go "idle"; it
    /// will maintain its internal state and await further instructions.
    /// When a HALT instruction is encountered, the VM's state will be
    /// cleared; it can be "restarted" by passing more instructions to
    /// execute.
    pub fn execute(&mut self, code: &Code) -> ExeResult {
        use Inst::*;

        let num_inst = code.len_chunk();
        let mut ip: usize = 0;
        let mut jump_ip = None;

        loop {
            match &code[ip] {
                NoOp => {
                    // do nothing
                }
                Pop => {
                    self.pop()?;
                }
                PopN(n) => {
                    self.pop_n(*n)?;
                }
                // Constants
                LoadGlobalConst(index) => {
                    self.push_global_const(*index)?;
                }
                LoadNil => {
                    self.push_global_const(0)?;
                }
                LoadTrue => {
                    self.push_global_const(1)?;
                }
                LoadFalse => {
                    self.push_global_const(2)?;
                }
                LoadConst(index) => {
                    self.push_const(code, *index)?;
                }
                // Scopes
                ScopeStart => {
                    self.enter_scope();
                }
                ScopeEnd => {
                    self.exit_scopes(1);
                }
                // Locals
                StoreLocal(index) => {
                    let obj = self.peek_obj()?;
                    self.value_stack[*index] = ValueStackKind::Local(obj, *index);
                }
                LoadLocal(index) => {
                    let kind = self.value_stack[*index].clone();
                    let obj = self.get_obj(&kind);
                    self.push_local(obj, *index);
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
                    self.push_var(depth, name.clone())?;
                }
                LoadVar(name) => {
                    let depth = self.ctx.get_var_depth(name.as_str())?;
                    self.push_var(depth, name.clone())?;
                }
                // Jumps
                Jump(addr, scope_exit_count) => {
                    self.exit_scopes(*scope_exit_count);
                    jump_ip = Some(*addr);
                }
                JumpPushNil(addr, scope_exit_count) => {
                    self.push_const(code, 0)?;
                    self.exit_scopes(*scope_exit_count);
                    jump_ip = Some(*addr);
                }
                JumpIf(addr, scope_exit_count) => {
                    self.exit_scopes(*scope_exit_count);
                    let obj = self.pop_obj()?;
                    let obj = obj.read().unwrap();
                    if obj.bool_val()? {
                        jump_ip = Some(*addr);
                    }
                }
                JumpIfNot(addr, scope_exit_count) => {
                    self.exit_scopes(*scope_exit_count);
                    let obj = self.pop_obj()?;
                    let obj = obj.read().unwrap();
                    if !obj.bool_val()? {
                        jump_ip = Some(*addr);
                    }
                }
                JumpIfElse(if_addr, else_addr, scope_exit_count) => {
                    self.exit_scopes(*scope_exit_count);
                    let obj = self.pop_obj()?;
                    let obj = obj.read().unwrap();
                    let addr = if obj.bool_val()? { *if_addr } else { *else_addr };
                    jump_ip = Some(addr);
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
                // Functions
                Call(n) => {
                    self.handle_call(*n)?;
                }
                Return => {
                    self.call_stack.pop();
                }
                // Object construction
                MakeString(n) => {
                    let objects = self.pop_n_obj(*n)?;
                    let mut string = String::with_capacity(32);
                    for obj in objects {
                        let obj = obj.read().unwrap();
                        string.push_str(obj.to_string().as_str());
                    }
                    let string_obj = create::new_str(string);
                    self.push_temp(string_obj);
                }
                MakeTuple(n) => {
                    let objects = self.pop_n_obj(*n)?;
                    let mut items = vec![];
                    for obj in objects {
                        items.push(obj.clone());
                    }
                    let tuple = create::new_tuple(items);
                    self.push_temp(tuple);
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
                Halt(return_code) => {
                    self.halt();
                    break Ok(VMState::Halted(*return_code));
                }
                HaltTop => {
                    let obj = self.pop_obj()?;
                    let obj = obj.read().unwrap();
                    let return_code = match obj.get_int_val() {
                        Some(int) => {
                            self.halt();
                            int.to_u8().unwrap_or(255)
                        }
                        None => 0,
                    };
                    break Ok(VMState::Halted(return_code));
                }
            }

            if let Some(new_ip) = jump_ip {
                ip = new_ip;
                jump_ip = None;
            } else {
                ip += 1;
            }

            if ip == num_inst {
                break Ok(VMState::Idle);
            }
        }
    }

    pub fn halt(&mut self) {
        // TODO: Not sure what this should do or if it's even needed
    }

    // Handlers --------------------------------------------------------

    fn handle_unary_op(&mut self, op: &UnaryOperator) -> RuntimeResult {
        use UnaryOperator::*;
        let a = self.pop_obj()?;
        let result = match op {
            Plus => a, // no-op
            Negate => a.read().unwrap().negate()?,
        };
        self.push_temp(result);
        Ok(())
    }

    fn handle_unary_compare_op(&mut self, op: &UnaryCompareOperator) -> RuntimeResult {
        use UnaryCompareOperator::*;
        let a = self.pop_obj()?;
        let a = a.read().unwrap();
        let result = match op {
            AsBool => a.bool_val()?,
            Not => a.not()?,
        };
        let obj = create::bool_obj_from_bool(result);
        self.push_temp(obj);
        Ok(())
    }

    /// Pop top two operands from stack, apply operation, and push temp
    /// result value onto stack.
    fn handle_binary_op(&mut self, op: &BinaryOperator) -> RuntimeResult {
        use BinaryOperator::*;
        let operands = self.pop_n_obj(2)?;
        let a_ref = operands.get(0).unwrap();
        let b_ref = operands.get(1).unwrap();
        let a = a_ref.read().unwrap();
        let b = b_ref.read().unwrap();
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
                let obj_ref = if let Some(name) = b.get_str_val() {
                    a.get_attr(name.as_str())?
                } else if let Some(index) = b.get_usize_val() {
                    a.get_item(index)?
                } else {
                    let message = format!("Not an attribute name or index: {b:?}");
                    return Err(RuntimeErr::new_type_err(message));
                };
                let bind = {
                    let obj = obj_ref.read().unwrap();
                    obj.is_builtin_func() || obj.is_func()
                };
                if bind {
                    // If `b` in `a.b` is a function, bind `b` to `a`.
                    create::new_bound_func(obj_ref.clone(), a_ref.clone())
                } else {
                    obj_ref
                }
            }
        };
        self.push_temp(result);
        Ok(())
    }

    /// Pop top two operands from stack, compare them, and push bool
    /// temp value onto stack.
    fn handle_compare_op(&mut self, op: &CompareOperator) -> RuntimeResult {
        use CompareOperator::*;
        let operands = self.pop_n_obj(2)?;
        let a_ref = operands.get(0).unwrap();
        let b_ref = operands.get(1).unwrap();
        let a = a_ref.read().unwrap();
        let b = b_ref.read().unwrap();
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
        self.push_temp(obj);
        Ok(())
    }

    /// Pop top two operands from stack, apply operation, assign result,
    /// and push temp result value onto stack. The first operand *must*
    /// be a variable.
    fn handle_inplace_op(&mut self, op: &InplaceOperator) -> RuntimeResult {
        use InplaceOperator::*;
        use ValueStackKind::*;
        let kinds = self.pop_n(2)?;
        let a_kind = kinds.get(0).unwrap();
        let b = self.get_obj(kinds.get(1).unwrap());
        let b = b.read().unwrap();
        if let Var(a, depth, name) = a_kind {
            let a = a.read().unwrap();
            let result = match op {
                AddEqual => a.add(&*b)?,
                SubEqual => a.sub(&*b)?,
            };
            self.ctx.assign_var_at_depth(*depth, name.as_str(), result)?;
            self.push_var(*depth, name.clone())?;
        } else if let Local(a, index) = a_kind {
            let a = a.read().unwrap();
            let result = match op {
                AddEqual => a.add(&*b)?,
                SubEqual => a.sub(&*b)?,
            };
            self.push_and_store_local(result, *index);
        } else {
            let message = format!("Binary op: {}", op);
            return Err(RuntimeErr::new(RuntimeErrKind::ExpectedVar(message)));
        }
        Ok(())
    }

    fn handle_call(&mut self, num_args: usize) -> RuntimeResult {
        let callable = self.pop_obj()?;
        let callable = callable.read().unwrap();
        let args = if num_args > 0 { self.pop_n_obj(num_args)? } else { vec![] };
        callable.call(args, self)
    }

    pub fn call_builtin_func(
        &mut self,
        func: &BuiltinFunc,
        this: This,
        args: Args,
    ) -> RuntimeResult {
        // NOTE: We create a call frame for builtin functions mainly
        //       just to check for too much recursion.
        self.push_call_frame()?;
        self.enter_scope();
        if this.is_some() {
            // NOTE: We assign `this` here so that if a user function is
            //       passed into a builtin function the user function
            //       can access it.
            let this_var = this.clone().unwrap().clone();
            self.ctx.declare_and_assign_var("this", this_var)?;
        }
        self.check_call_args(func.name.as_str(), &func.params, &args)?;
        let return_val = (func.func)(this, args, self)?;
        self.push_return_val(return_val);
        self.exit_scopes(1);
        // NOTE: We have to pop the frame for builtin functions since
        //       they don't exit with a RETURN instruction like user
        //       functions.
        self.pop_call_frame()?;
        Ok(())
    }

    pub fn call_func(&mut self, func: &Func, this: This, args: Args) -> RuntimeResult {
        self.push_call_frame()?;
        self.enter_scope();
        if let Some(this_var) = this {
            self.ctx.declare_and_assign_var("this", this_var)?;
        }
        self.check_call_args(func.name.as_str(), &func.params, &args)?;
        self.execute(&func.code)?;
        self.exit_scopes(1);
        Ok(())
    }

    /// Check call args to ensure they're valid. If they are, bind them
    /// to names in the call scope (if `bind` is specified).
    pub fn check_call_args(
        &mut self,
        name: &str,
        params: &Params,
        args: &Args,
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
            for (name, arg) in params.iter().zip(args) {
                self.ctx.declare_and_assign_var(name, arg.clone())?;
            }
        } else {
            let args = create::new_tuple(args.clone());
            self.ctx.declare_and_assign_var("$args", args)?;
        }
        Ok(())
    }

    // Scopes ----------------------------------------------------------

    pub fn enter_scope(&mut self) {
        self.ctx.enter_scope();
    }

    /// When exiting a scope, we first ensure the scope left a result
    /// on the stack (or, at least, that the stack isn't empty) and then
    /// clear the scope's namespace.
    pub fn exit_scopes(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        // Ensure the scope left a value on the stack.
        if self.peek().is_err() {
            panic!("Value stack unexpectedly empty when exiting scope(s): {count}");
        }
        self.ctx.exit_scopes(count);
    }

    // Value stack -----------------------------------------------------

    fn push(&mut self, kind: ValueStackKind) {
        self.value_stack.push(kind);
    }

    fn push_global_const(&mut self, index: usize) -> RuntimeResult {
        let obj = self.ctx.get_global_const(index)?.clone();
        self.push(ValueStackKind::GlobalConstant(obj, index));
        Ok(())
    }

    fn push_const(&mut self, code: &Code, index: usize) -> RuntimeResult {
        let obj = code.get_const(index)?.clone();
        self.push(ValueStackKind::Constant(obj, index));
        Ok(())
    }

    fn push_local(&mut self, obj: ObjectRef, index: usize) {
        self.push(ValueStackKind::Local(obj, index));
    }

    fn push_and_store_local(&mut self, obj: ObjectRef, index: usize) {
        self.value_stack[index] = ValueStackKind::Local(obj.clone(), index);
        self.push_local(obj, index);
    }

    fn push_var(&mut self, depth: usize, name: String) -> RuntimeResult {
        let obj = self.ctx.get_var_at_depth(depth, name.as_str())?;
        self.push(ValueStackKind::Var(obj, depth, name));
        Ok(())
    }

    fn push_temp(&mut self, obj: ObjectRef) {
        self.push(ValueStackKind::Temp(obj));
    }

    fn push_return_val(&mut self, obj: ObjectRef) {
        self.push(ValueStackKind::ReturnVal(obj));
    }

    fn pop(&mut self) -> PopResult {
        match self.value_stack.pop() {
            Some(kind) => Ok(kind),
            None => Err(RuntimeErr::new_empty_statck()),
        }
    }

    fn pop_obj(&mut self) -> PopObjResult {
        let kind = self.pop()?;
        Ok(self.get_obj(&kind))
    }

    fn pop_n(&mut self, n: usize) -> PopNResult {
        match self.value_stack.pop_n(n) {
            Some(kinds) => Ok(kinds),
            None => Err(RuntimeErr::new_not_enough_values_on_stack(n)),
        }
    }

    fn pop_n_obj(&mut self, n: usize) -> PopNObjResult {
        let kinds = self.pop_n(n)?;
        let objects = kinds.iter().map(|k| self.get_obj(k)).collect();
        Ok(objects)
    }

    fn peek(&self) -> PeekResult {
        match self.value_stack.peek() {
            Some(kind) => Ok(kind),
            None => Err(RuntimeErr::new_empty_statck()),
        }
    }

    pub fn peek_obj(&mut self) -> PeekObjResult {
        let kind = self.peek()?;
        Ok(self.get_obj(kind))
    }

    fn get_obj(&self, kind: &ValueStackKind) -> ObjectRef {
        use ValueStackKind::*;
        match kind {
            GlobalConstant(obj, ..) => obj.clone(),
            Constant(obj, ..) => obj.clone(),
            Var(obj, ..) => obj.clone(),
            Local(obj, ..) => obj.clone(),
            Temp(obj) => obj.clone(),
            ReturnVal(obj) => obj.clone(),
        }
    }

    // Call Stack ------------------------------------------------------

    fn push_call_frame(&mut self) -> RuntimeResult {
        if self.call_stack.size() == self.max_call_depth {
            return Err(RuntimeErr::new_recursion_depth_exceeded(self.max_call_depth));
        }
        let stack_position = self.value_stack.size();
        let frame = CallFrame::new(stack_position);
        self.call_stack.push(frame);
        Ok(())
    }

    fn pop_call_frame(&mut self) -> Result<CallFrame, RuntimeErr> {
        match self.call_stack.pop() {
            Some(frame) => Ok(frame),
            None => Err(RuntimeErr::new(RuntimeErrKind::EmptyCallStack)),
        }
    }

    // Utilities -------------------------------------------------------

    /// Show the contents of the stack (top first).
    pub fn display_stack(&self) {
        if self.value_stack.is_empty() {
            return eprintln!("[EMPTY]");
        }
        for (i, kind) in self.value_stack.iter().enumerate() {
            let obj = self.get_obj(kind);
            let obj = &*obj.read().unwrap();
            eprintln!("{:0>8} {:?}", i, obj);
        }
    }
}
