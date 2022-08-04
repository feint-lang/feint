//! The FeInt virtual machine. When it's created, it's initialized and
//! then, implicitly, goes idle until it's passed some instructions to
//! execute. After instructions are executed, it goes back into idle
//! mode.
use std::fmt;

use num_traits::ToPrimitive;

use crate::types::{
    create, Args, BuiltinFunc, Func, ObjectRef, ObjectTraitExt, Params, This,
};
use crate::util::{
    BinaryOperator, CompareOperator, InplaceOperator, Stack, UnaryCompareOperator,
    UnaryOperator,
};

use super::code::Code;
use super::context::RuntimeContext;
use super::inst::Inst;
use super::result::{
    CallDepth, ExeResult, PeekObjResult, PopNObjResult, PopObjResult, RuntimeErr,
    RuntimeErrKind, RuntimeResult, VMState,
};

pub const DEFAULT_MAX_CALL_DEPTH: CallDepth =
    if cfg!(debug_assertions) { 256 } else { 1024 };

#[derive(Clone, Debug)]
enum ValueStackKind {
    GlobalConstant(ObjectRef, usize),
    Constant(ObjectRef, usize),
    Var(ObjectRef, usize, String),
    Temp(ObjectRef),
    ReturnVal(ObjectRef),
}

struct CallFrame {
    pub _stack_position: usize,
    pub locals: Vec<ObjectRef>,
}

impl CallFrame {
    pub fn new(_stack_position: usize, locals: Vec<ObjectRef>) -> Self {
        Self { _stack_position, locals }
    }

    pub fn set_local(&mut self, index: usize, obj: ObjectRef) {
        self.locals[index] = obj;
    }
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
        VM {
            ctx,
            value_stack: Stack::new(),
            scope_stack: Stack::new(),
            call_stack: Stack::new(),
            max_call_depth,
        }
    }

    /// Execute the specified instructions and return the VM's state. If
    /// a HALT instruction isn't encountered, the VM will go "idle"; it
    /// will maintain its internal state and await further instructions.
    /// When a HALT instruction is encountered, the VM's state will be
    /// cleared; it can be "restarted" by passing more instructions to
    /// execute.
    pub fn execute(&mut self, code: &Code, dis: bool) -> ExeResult {
        use Inst::*;

        let num_inst = code.len_chunk();
        let mut ip: usize = 0;
        let mut jump_ip = None;

        loop {
            match &code[ip] {
                NoOp => {
                    // do nothing
                }
                Truncate(size) => {
                    self.value_stack.truncate(*size);
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
                StoreLocal(index) => {
                    let obj = self.pop_obj()?;
                    // XXX: We have to pop the frame and put it back in
                    //      order to mutate it. There's probably a
                    //      better way to handle this.
                    let mut frame = self.pop_call_frame()?;
                    frame.set_local(*index, obj);
                    self.call_stack.push(frame);
                }
                LoadLocal(index) => {
                    let frame = self.call_stack.peek().expect("Empty call stack");
                    let obj = frame.locals[*index].clone();
                    self.push_temp(obj);
                }
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
                    if dis {
                        self.dis(ip, code);
                    }
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
                    if dis {
                        self.dis(ip, code);
                    }
                    break Ok(VMState::Halted(return_code));
                }
            }

            if dis {
                self.dis(ip, code);
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
        if let Some(kinds) = self.pop_n(2) {
            if let ValueStackKind::Var(a, depth, name) = kinds.get(0).unwrap() {
                let b = self.get_obj(kinds.get(1).unwrap());
                let a = a.read().unwrap();
                let b = b.read().unwrap();
                let result = match op {
                    AddEqual => a.add(&*b)?,
                    SubEqual => a.sub(&*b)?,
                };
                self.ctx.assign_var_at_depth(*depth, name.as_str(), result)?;
                self.push_var(*depth, name.clone())?;
            } else {
                let message = format!("Binary op: {}", op);
                return Err(RuntimeErr::new(RuntimeErrKind::ExpectedVar(message)));
            }
        } else {
            return Err(RuntimeErr::new(RuntimeErrKind::NotEnoughValuesOnStack(2)));
        }
        Ok(())
    }

    fn handle_call(&mut self, num_args: usize) -> RuntimeResult {
        let callable = self.pop_obj()?;
        let callable = callable.read().unwrap();
        let args = if num_args > 0 { self.pop_n_obj(num_args)? } else { vec![] };
        callable.call(args, self)
    }

    fn push_call_frame(&mut self, args: &Args) -> RuntimeResult {
        if self.call_stack.size() == self.max_call_depth {
            return Err(RuntimeErr::new_recursion_depth_exceeded(self.max_call_depth));
        }
        let stack_position = self.value_stack.size();
        let frame = CallFrame::new(stack_position, args.clone());
        self.call_stack.push(frame);
        Ok(())
    }

    fn pop_call_frame(&mut self) -> Result<CallFrame, RuntimeErr> {
        match self.call_stack.pop() {
            Some(frame) => Ok(frame),
            None => Err(RuntimeErr::new(RuntimeErrKind::EmptyCallStack)),
        }
    }

    pub fn call_builtin_func(
        &mut self,
        func: &BuiltinFunc,
        this: This,
        args: Args,
    ) -> RuntimeResult {
        // NOTE: We create a call frame for builtin functions mainly
        //       just to check for too much recursion. Builtin functions
        //       don't use frame locals, so they're not passed in.
        self.push_call_frame(&vec![])?;
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
        self.push_call_frame(&args)?;
        self.enter_scope();
        if let Some(this_var) = this {
            self.ctx.declare_and_assign_var("this", this_var)?;
        }
        self.check_call_args(func.name.as_str(), &func.params, &args)?;
        self.execute(&func.code, false)?;
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

    pub fn enter_scope(&mut self) {
        self.scope_stack.push(self.value_stack.size());
        self.ctx.enter_scope();
    }

    /// When exiting a scope, we first save the top of the stack (which
    /// is the "return value" of the scope), remove all stack values
    /// added in the scope, and finally push the scope's "return value"
    /// back onto the stack. After taking care of the VM stack, the
    /// scope's namespace is then cleared and removed.
    pub fn exit_scopes(&mut self, count: usize) {
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
            self.push_temp(obj);
        } else {
            panic!("Value stack unexpectedly empty when exiting scope(s): {count}");
        }
        self.ctx.exit_scopes(count);
    }

    pub fn halt(&mut self) {
        // TODO: Not sure what this should do or if it's even needed
    }

    // Value stack -----------------------------------------------------

    fn push(&mut self, kind: ValueStackKind) {
        self.value_stack.push(kind);
    }

    pub fn push_global_const(&mut self, index: usize) -> RuntimeResult {
        let obj = self.ctx.get_global_const(index)?.clone();
        self.push(ValueStackKind::GlobalConstant(obj, index));
        Ok(())
    }

    pub fn push_const(&mut self, code: &Code, index: usize) -> RuntimeResult {
        let obj = code.get_const(index)?.clone();
        self.push(ValueStackKind::Constant(obj, index));
        Ok(())
    }

    pub fn push_return_val(&mut self, obj: ObjectRef) {
        self.push(ValueStackKind::ReturnVal(obj));
    }

    pub fn push_temp(&mut self, obj: ObjectRef) {
        self.push(ValueStackKind::Temp(obj));
    }

    fn push_var(&mut self, depth: usize, name: String) -> RuntimeResult {
        let obj = self.ctx.get_var_at_depth(depth, name.as_str())?;
        self.push(ValueStackKind::Var(obj, depth, name));
        Ok(())
    }

    fn pop(&mut self) -> Option<ValueStackKind> {
        self.value_stack.pop()
    }

    pub fn pop_obj(&mut self) -> PopObjResult {
        match self.pop() {
            Some(kind) => Ok(self.get_obj(&kind)),
            None => Err(RuntimeErr::new(RuntimeErrKind::EmptyStack)),
        }
    }

    fn pop_n(&mut self, n: usize) -> Option<Vec<ValueStackKind>> {
        self.value_stack.pop_n(n)
    }

    fn pop_n_obj(&mut self, n: usize) -> PopNObjResult {
        match self.pop_n(n) {
            Some(kinds) => Ok(kinds.iter().map(|k| self.get_obj(k)).collect()),
            None => Err(RuntimeErr::new(RuntimeErrKind::NotEnoughValuesOnStack(n))),
        }
    }

    fn peek(&self) -> Option<&ValueStackKind> {
        self.value_stack.peek()
    }

    pub fn peek_obj(&mut self) -> PeekObjResult {
        self.peek().map(|kind| self.get_obj(kind))
    }

    fn get_obj(&self, kind: &ValueStackKind) -> ObjectRef {
        use ValueStackKind::*;
        match kind {
            GlobalConstant(obj, ..) => obj.clone(),
            Constant(obj, ..) => obj.clone(),
            Var(obj, ..) => obj.clone(),
            Temp(obj) => obj.clone(),
            ReturnVal(obj) => obj.clone(),
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

    /// Show constants.
    pub fn display_constants(&self) {
        for (index, obj) in self.ctx.iter_constants().enumerate() {
            let obj = obj.read().unwrap();
            eprintln!("{index:0>8} {obj}");
        }
    }

    /// Show vars.
    pub fn display_vars(&self) {
        let mut entries = self.ctx.iter_vars().collect::<Vec<_>>();
        entries.sort_by_key(|(n, _)| n.as_str());
        for (name, obj) in entries.iter() {
            let obj = obj.read().unwrap();
            if obj.module_name() != "builtins" {
                eprintln!("{name} = {obj}");
            }
        }
    }

    // Disassembler ----------------------------------------------------
    //
    // This is done here because we need the VM context in order to show
    // more useful info like jump targets, values, etc.

    /// Disassemble a list of instructions.
    pub fn dis_list(&mut self, code: &Code) -> ExeResult {
        for (ip, _) in code.iter_inst().enumerate() {
            self.dis(ip, code);
        }
        Ok(VMState::Halted(0))
    }

    /// Disassemble functions, returning the number of functions that
    /// were disassembled.
    pub fn dis_functions(&mut self, code: &Code) -> usize {
        let mut funcs = vec![];
        for obj_ref in code.iter_constants() {
            let obj = obj_ref.read().unwrap();
            let is_func = obj.down_to_func().is_some();
            if is_func {
                funcs.push(obj_ref.clone());
            }
        }
        let num_funcs = funcs.len();
        for (i, func_obj) in funcs.iter().enumerate() {
            let func_obj = func_obj.read().unwrap();
            let func = func_obj.down_to_func().unwrap();
            let heading = format!("{func:?} ");
            eprintln!("{:=<79}", heading);
            if let Err(err) = self.dis_list(&func.code) {
                eprintln!("Could not disassemble function {func}: {err}");
            }
            if num_funcs > 1 && i < (num_funcs - 1) {
                eprintln!();
            }
        }
        num_funcs
    }

    /// Disassemble a single instruction.
    pub fn dis(&mut self, ip: usize, code: &Code) {
        let inst = &code[ip];
        let formatted = self.format_instruction(code, inst);
        eprintln!("{:0>4} {}", ip, formatted);
    }

    fn global_const_str(&self, index: usize) -> String {
        match self.ctx.get_global_const(index) {
            Ok(obj) => self.obj_str(obj.clone()),
            Err(err) => format!("[COULD NOT LOAD GLOBAL CONSTANT AT {index}: {err}]"),
        }
    }

    fn const_str(&self, code: &Code, index: usize) -> String {
        match code.get_const(index) {
            Ok(obj) => self.obj_str(obj.clone()),
            Err(err) => format!("[COULD NOT LOAD CONSTANT AT {index}: {err}]"),
        }
    }

    fn var_str(&mut self, name: &str) -> String {
        match self.ctx.get_var(name) {
            Ok(obj) => self.obj_str(obj.clone()),
            Err(err) => format!("[COULD NOT LOAD VAR {name}: {err}]"),
        }
    }

    fn obj_str(&self, obj: ObjectRef) -> String {
        let obj = &*obj.read().unwrap();
        let str = format!("{obj:?} {}", obj.class().read().unwrap());
        str.replace('\n', "\\n").replace('\r', "\\r")
    }

    fn format_instruction(&mut self, code: &Code, inst: &Inst) -> String {
        use Inst::*;
        match inst {
            NoOp => "NOOP".to_string(),
            Truncate(size) => self.format_aligned("TRUNCATE", format!("{size}")),
            LoadGlobalConst(index) => {
                let obj_str = self.global_const_str(*index);
                self.format_aligned("LOAD_GLOBAL_CONST", format!("{index} : {obj_str}"))
            }
            LoadNil => "LOAD_NIL".to_string(),
            LoadTrue => "LOAD_TRUE".to_string(),
            LoadFalse => "LOAD_FALSE".to_string(),
            LoadConst(index) => {
                let obj_str = self.const_str(code, *index);
                self.format_aligned("LOAD_CONST", format!("{index} : {obj_str}"))
            }
            ScopeStart => "SCOPE_START".to_string(),
            ScopeEnd => "SCOPE_END".to_string(),
            DeclareVar(name) => self.format_aligned("DECLARE_VAR", name),
            AssignVar(name) => {
                let obj_str = self.var_str(name);
                self.format_aligned("ASSIGN_VAR", format!("{name} = {obj_str}"))
            }
            LoadVar(name) => {
                let obj_str = self.var_str(name);
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
            StoreLocal(index) => {
                let obj = self.peek_obj().unwrap();
                let obj_str = self.obj_str(obj);
                self.format_aligned("STORE_LOCAL", format!("{index} : {obj_str}"))
            }
            LoadLocal(index) => {
                let obj_str = if let Some(frame) = self.call_stack.peek() {
                    let obj = frame.locals.get(*index).unwrap();
                    self.obj_str(obj.clone())
                } else {
                    "Empty call stack".to_string()
                };
                self.format_aligned("LOAD_LOCAL", format!("{index} : {obj_str}"))
            }
            Call(n) => self.format_aligned("CALL", n),
            Return => "RETURN".to_string(),
            MakeString(n) => self.format_aligned("MAKE_STRING", n),
            MakeTuple(n) => self.format_aligned("MAKE_TUPLE", n),
            Halt(code) => self.format_aligned("HALT", code),
            HaltTop => {
                if let Some(obj) = self.peek_obj() {
                    let obj = obj.read().unwrap();
                    self.format_aligned("HALT_TOP", obj.to_string())
                } else {
                    self.format_aligned("HALT_TOP", "[ERROR: empty stack]")
                }
            }
            Placeholder(addr, inst, message) => {
                let formatted_inst = self.format_instruction(code, inst);
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
