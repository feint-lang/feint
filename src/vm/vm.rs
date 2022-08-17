//! The FeInt virtual machine. When it's created, it's initialized and
//! then, implicitly, goes idle until it's passed some instructions to
//! execute. After instructions are executed, it goes back into idle
//! mode.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use ctrlc;
use num_traits::ToPrimitive;

use crate::modules;
use crate::types::{args_to_str, this_to_str};
use crate::types::{
    create, Args, BuiltinFunc, Func, FuncTrait, ObjectRef, ObjectTrait, This,
};
use crate::util::{
    BinaryOperator, CompareOperator, InplaceOperator, Location, Stack,
    UnaryCompareOperator, UnaryOperator,
};
use crate::vm::result::PeekResult;

use super::code::Code;
use super::context::RuntimeContext;
use super::inst::Inst;
use super::result::{
    CallDepth, PeekObjResult, PopNObjResult, PopNResult, PopObjResult, PopResult,
    RuntimeErr, RuntimeObjResult, RuntimeResult, VMExeResult, VMState, ValueStackKind,
};

pub const DEFAULT_MAX_CALL_DEPTH: CallDepth =
    if cfg!(debug_assertions) { 256 } else { 1024 };

struct CallFrame {
    stack_pointer: usize,
    closure: Option<ObjectRef>,
}

impl CallFrame {
    pub fn new(stack_pointer: usize, closure: Option<ObjectRef>) -> Self {
        Self { stack_pointer, closure }
    }

    pub fn get_cell_data(&self, index: usize) -> RuntimeObjResult {
        if let Some(closure) = &self.closure {
            let closure = closure.read().unwrap();
            let closure = closure.down_to_closure().unwrap();
            let cells = closure.cells.read().unwrap();
            if let Some(obj) = cells.get(index) {
                return Ok(obj.clone());
            }
        }
        Err(RuntimeErr::cell_not_found(index))
    }

    pub fn set_cell_data(&self, index: usize, data: ObjectRef) -> RuntimeResult {
        if let Some(closure) = &self.closure {
            let closure = closure.read().unwrap();
            let closure = closure.down_to_closure().unwrap();
            let mut cells = closure.cells.write().unwrap();
            cells[index] = data;
        }
        Err(RuntimeErr::cell_not_found(index))
    }
}

pub struct VM {
    ctx: RuntimeContext,
    // The scope stack contains pointers into the value stack. When a
    // scope is entered, the current, pre-scope stack position is
    // recorded. When a scope is exited, its corresponding pointer is
    // used to truncate the value stack, removing all temporaries and
    // locals introduced by the scope.
    scope_stack: Stack<usize>,
    // The value stack contains "pointers" to the different value types:
    // constants, vars, temporaries, and return values.
    value_stack: Stack<ValueStackKind>,
    // Call stack. We manually track the size to avoid calling len().
    call_stack: Stack<CallFrame>,
    call_stack_size: usize,
    call_frame_pointer: usize,
    // Maximum depth of "call stack" (quotes because there's no explicit
    // call stack).
    max_call_depth: CallDepth,
    // The location of the current statement. Used for error reporting.
    loc: (Location, Location),
    // SIGINT (Ctrl-C) handling.
    handle_sigint: bool, // whether the VM should handle SIGINT
    sigint_flag: Arc<AtomicBool>, // indicates SIGINT was sent
}

unsafe impl Send for VM {}
unsafe impl Sync for VM {}

impl Default for VM {
    fn default() -> Self {
        VM::new(RuntimeContext::new(), DEFAULT_MAX_CALL_DEPTH)
    }
}

impl VM {
    pub fn new(ctx: RuntimeContext, max_call_depth: CallDepth) -> Self {
        let sigint_flag = Arc::new(AtomicBool::new(false));
        VM {
            ctx,
            scope_stack: Stack::with_capacity(max_call_depth),
            value_stack: Stack::with_capacity(max_call_depth * 8),
            call_stack: Stack::with_capacity(max_call_depth),
            call_stack_size: 0,
            call_frame_pointer: 0,
            max_call_depth,
            loc: (Location::default(), Location::default()),
            handle_sigint: sigint_flag.load(Ordering::Relaxed),
            sigint_flag,
        }
    }

    /// Execute the specified instructions and return the VM's state. If
    /// a HALT instruction isn't encountered, the VM will go "idle"; it
    /// will maintain its internal state and await further instructions.
    /// When a HALT instruction is encountered, the VM's state will be
    /// cleared; it can be "restarted" by passing more instructions to
    /// execute.
    pub fn execute(&mut self, code: &Code) -> VMExeResult {
        use Inst::*;

        let handle_sigint = self.handle_sigint;
        let sigint_flag = self.sigint_flag.clone();
        let mut sigint_counter = 0u32;

        let num_inst = code.len_chunk();
        let mut ip: usize = 0;
        let mut jump_ip = None;

        loop {
            match &code[ip] {
                DisplayStack(message) => {
                    eprintln!("\nSTACK: {message}\n");
                    self.display_stack();
                    eprintln!();
                }
                NoOp => {
                    // do nothing
                }
                Pop => {
                    if let ValueStackKind::Local(..) = self.peek()? {
                        // Locals are cleaned up separately
                    } else {
                        self.pop()?;
                    }
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
                // Scopes
                FuncScopeStart(args_offset) => {
                    self.enter_scope(*args_offset);
                }
                ScopeStart => {
                    self.enter_scope(0);
                }
                ScopeEnd => {
                    self.exit_scope();
                }
                StatementStart(start, end) => {
                    self.loc = (*start, *end);
                }
                LoadConst(index) => {
                    self.push_const(code, *index)?;
                }
                // Locals
                StoreLocal(index) => {
                    // Store object at TOS into local slot. Note that
                    // this leaves TOS in place.
                    let obj = self.peek_obj()?;
                    self.store_local(obj, *index)?;
                }
                // This called from an ancestor frame. It needs access
                // to the cell storage of the closure(s) it was added
                // for.
                StoreLocalAndCell(local_index, cell_index) => {
                    // Store object at TOS into local slot and to cell.
                    let obj = self.peek_obj()?;
                    self.store_local(obj.clone(), *local_index)?;
                    // let frame = self.current_call_frame()?;
                    // frame.set_cell_data(*cell_index, obj.clone())?;
                }
                LoadLocal(index) => {
                    // Load (copy) specified local to TOS.
                    self.load_local(*index)?;
                }
                LoadCell(index) => {
                    // Load data from cell to TOS.
                    let frame = self.current_call_frame()?;
                    let obj = frame.get_cell_data(*index)?;
                    self.push_temp(obj);
                }
                // Args (locals for Call)
                ToArg(local_index) => {
                    log::trace!("BEFORE CONVERSION TO ARG:\n{}", self.format_stack());
                    let offset = local_index + 1;
                    if offset > self.value_stack.len() {
                        panic!("Arg offset out of bounds: {offset}");
                    }
                    let stack_index = self.value_stack.len() - offset;
                    if let Ok(kind) = self.peek_at(stack_index) {
                        let arg = self.get_obj(kind);
                        log::trace!("ARG: {arg:?}");
                        let local = ValueStackKind::Local(arg, *local_index);
                        self.replace(stack_index, local)?;
                    } else {
                        panic!("Expected stack value at {stack_index}");
                    }
                    log::trace!("AFTER CONVERSION TO ARG:\n{}", self.format_stack());
                }
                ToArgAndCell(local_index, cell_index) => {
                    log::trace!(
                        "BEFORE CONVERSION TO ARG AND CELL:\n{}",
                        self.format_stack()
                    );
                    let offset = local_index + 1;
                    if offset > self.value_stack.len() {
                        panic!("Arg offset out of bounds: {offset}");
                    }
                    let stack_index = self.value_stack.len() - offset;
                    if let Ok(kind) = self.peek_at(stack_index) {
                        let arg = self.get_obj(kind);
                        log::trace!("ARG: {arg:?}");
                        let local = ValueStackKind::Local(arg, *local_index);
                        self.replace(stack_index, local)?;
                    } else {
                        panic!("Expected stack value at {stack_index}");
                    }
                    log::trace!("AFTER CONVERSION TO ARG:\n{}", self.format_stack());
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
                Jump(addr, forward, scope_exit_count) => {
                    self.exit_scopes(*scope_exit_count);
                    if *forward {
                        jump_ip = Some(ip + *addr);
                    } else {
                        jump_ip = Some(ip - *addr);
                    }
                }
                JumpPushNil(addr, forward, scope_exit_count) => {
                    self.push_global_const(0)?;
                    self.exit_scopes(*scope_exit_count);
                    if *forward {
                        jump_ip = Some(ip + *addr);
                    } else {
                        jump_ip = Some(ip - *addr);
                    }
                }
                JumpIfNot(addr, forward, scope_exit_count) => {
                    self.exit_scopes(*scope_exit_count);
                    let obj = self.peek_obj()?;
                    let obj = obj.read().unwrap();
                    if !obj.bool_val()? {
                        if *forward {
                            jump_ip = Some(ip + *addr);
                        } else {
                            jump_ip = Some(ip - *addr);
                        }
                    }
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
                    self.exit_scope();
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
                    let tuple = create::new_tuple(objects);
                    self.push_temp(tuple);
                }
                MakeList(n) => {
                    let objects = self.pop_n_obj(*n)?;
                    let list = create::new_list(objects);
                    self.push_temp(list);
                }
                MakeMap(n) => {
                    let objects = self.pop_n_obj(*n)?;
                    let mut keys = vec![];
                    let mut vals = vec![];
                    for (i, obj) in objects.iter().enumerate() {
                        if i % 2 == 0 {
                            let obj = obj.read().unwrap();
                            let key = obj.to_string();
                            keys.push(key);
                        } else {
                            vals.push(obj.clone());
                        }
                    }
                    let entries: Vec<(String, ObjectRef)> =
                        keys.into_iter().zip(vals).collect();
                    let map = create::new_map(entries);
                    self.push_temp(map);
                }
                // A closure is created in its parent frame.
                MakeClosure(func_const_index, cell_info) => {
                    let func = code.get_const(*func_const_index)?.clone();
                    let mut cells = vec![];
                    for (call_stack_index, local_index) in cell_info {
                        let frame =
                            &self.call_stack[self.call_stack_size - call_stack_index];

                        let index = frame.stack_pointer + local_index;

                        if let Ok(kind) = self.peek_at(index) {
                            assert!(matches!(kind, ValueStackKind::Local(..)));
                            let obj = self.peek_at_obj(index)?;
                            cells.push(obj);
                        } else {
                            // local not stored yet
                            cells.push(create::new_nil());
                        }
                    }
                    let closure = create::new_closure(func, cells);
                    self.push_temp(closure);
                }
                // Modules
                LoadModule(name) => {
                    let module = modules::get_module(name.as_str())?;
                    self.ctx.declare_and_assign_var(name, module.clone())?;
                    self.push_temp(module.clone());
                }
                // Placeholders
                Placeholder(addr, inst, message) => {
                    self.halt();
                    eprintln!(
                        "Placeholder at {addr} was not updated: {inst:?}\n{message}"
                    );
                    break Ok(VMState::Halted(255));
                }
                VarPlaceholder(addr, name) => {
                    self.halt();
                    eprintln!("Var placeholder at {addr} was not updated: {name}");
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
                ReturnPlaceholder(addr, _) => {
                    self.halt();
                    eprintln!("Return placeholder at {addr} was not updated");
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

            if handle_sigint {
                sigint_counter += 1;
                // TODO: Maybe use a different value and/or make it
                //       configurable.
                if sigint_counter == 1024 {
                    if sigint_flag.load(Ordering::Relaxed) {
                        self.handle_sigint();
                        break Ok(VMState::Idle);
                    }
                    sigint_counter = 0;
                }
            }

            if ip == num_inst {
                break Ok(VMState::Idle);
            }
        }
    }

    pub fn halt(&mut self) {
        // TODO: Not sure what this should do or if it's even needed
    }

    /// Get location of current statement (start and end).
    pub fn loc(&self) -> (Location, Location) {
        self.loc
    }

    pub fn install_sigint_handler(&mut self) {
        let flag = self.sigint_flag.clone();
        self.handle_sigint = true;
        if let Err(err) = ctrlc::set_handler(move || {
            flag.store(true, Ordering::Relaxed);
        }) {
            eprintln!("Could not install SIGINT handler: {err}");
        }
    }

    fn handle_sigint(&mut self) {
        self.sigint_flag.store(false, Ordering::Relaxed);
        self.reset();
    }

    /// Reset internal state.
    fn reset(&mut self) {
        self.scope_stack.truncate(0);
        self.value_stack.truncate(0);
        self.call_stack.truncate(0);
        self.call_stack_size = 0;
        self.call_frame_pointer = 0;
        self.ctx.exit_all_scopes();
    }

    // Handlers --------------------------------------------------------

    fn handle_unary_op(&mut self, op: &UnaryOperator) -> RuntimeResult {
        use UnaryOperator::*;
        let a_kind = self.pop()?;
        let a_ref = self.get_obj(&a_kind);
        let result = match op {
            Plus => a_ref, // no-op
            Negate => a_ref.read().unwrap().negate()?,
        };
        self.push_temp(result);
        Ok(())
    }

    fn handle_unary_compare_op(&mut self, op: &UnaryCompareOperator) -> RuntimeResult {
        use UnaryCompareOperator::*;
        let a_kind = self.pop()?;
        let a_ref = self.get_obj(&a_kind);
        let a = a_ref.read().unwrap();
        let result = match op {
            AsBool => a.bool_val()?,
            Not => a.not()?,
        };
        self.push_temp(create::new_bool(result));
        Ok(())
    }

    /// Pop top two operands from stack, apply operation, and push temp
    /// result value onto stack.
    fn handle_binary_op(&mut self, op: &BinaryOperator) -> RuntimeResult {
        use BinaryOperator::*;
        let b_ref = self.pop_obj()?;
        let a_kind = self.pop()?;
        let a_ref = self.get_obj(&a_kind);
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
                    a.get_attr(name)?
                } else if let Some(index) = b.get_usize_val() {
                    a.get_item(index)?
                } else {
                    let message = format!("Not an attribute name or index: {b:?}");
                    return Err(RuntimeErr::type_err(message));
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
        // Get RHS (b) first because we need to know if LHS (a) is a local
        let b_ref = self.pop_obj()?;
        let a_kind = self.pop()?;
        let a_ref = self.get_obj(&a_kind);
        let a = a_ref.read().unwrap();
        let b = b_ref.read().unwrap();
        let b = &*b;
        let result = match op {
            Is => a.is(b),
            IsNot => !a.is(b),
            IsEqual => a.is_equal(b),
            NotEqual => a.not_equal(b),
            And => a.and(b)?,
            Or => a.or(b)?,
            LessThan => a.less_than(b)?,
            LessThanOrEqual => a.less_than(b)? || a.is_equal(b),
            GreaterThan => a.greater_than(b)?,
            GreaterThanOrEqual => a.greater_than(b)? || a.is_equal(b),
        };
        self.push_temp(create::new_bool(result));
        Ok(())
    }

    /// Pop top two operands from stack, apply operation, assign result,
    /// and push temp result value onto stack. The first operand *must*
    /// be a variable.
    fn handle_inplace_op(&mut self, op: &InplaceOperator) -> RuntimeResult {
        let b_ref = self.pop_obj()?;
        let a_kind = self.pop()?;
        let a_ref = self.get_obj(&a_kind);
        let a = a_ref.read().unwrap();
        let b = b_ref.read().unwrap();
        let b = &*b;
        let result = match op {
            InplaceOperator::Mul => a.mul(&*b)?,
            InplaceOperator::Div => a.div(&*b)?,
            InplaceOperator::Add => a.add(&*b)?,
            InplaceOperator::Sub => a.sub(&*b)?,
        };
        if let ValueStackKind::Var(_, depth, name) = a_kind {
            self.ctx.assign_var_at_depth(depth, name.as_str(), result.clone())?;
            self.push_temp(result);
        } else if let ValueStackKind::TempLocal(_, index) = a_kind {
            log::trace!("STORE LOCAL FOR INPLACE {index}");
            self.store_local(result.clone(), index)?;
            self.push_temp(result);
        } else {
            return Err(RuntimeErr::expected_var(format!("Binary op: {}", op)));
        }
        Ok(())
    }

    fn handle_call(&mut self, num_args: usize) -> RuntimeResult {
        let callable_ref = self.pop_obj()?;
        let callable = callable_ref.read().unwrap();
        log::trace!("HANDLE CALL: callable = {:?}", &*callable);
        if let Some(func) = callable.down_to_builtin_func() {
            let args = if num_args > 0 { self.pop_n_obj(num_args)? } else { vec![] };
            self.call_builtin_func(func, None, args)
        } else if let Some(func) = callable.down_to_func() {
            log::trace!("CALL FUNC: {}\n{}", func.name(), self.format_stack());
            self.call_func(func, num_args)
        } else if let Some(closure) = callable.down_to_closure() {
            log::trace!("CALL CLOSURE: {}\n{}", closure.name(), self.format_stack());
            self.call_closure(callable_ref.clone(), num_args)
        } else if let Some(func) = callable.down_to_bound_func() {
            let args = if num_args > 0 { self.pop_n_obj(num_args)? } else { vec![] };
            func.call(args, self)
        } else {
            Err(callable.not_callable())
        }
    }

    pub fn call_builtin_func(
        &mut self,
        func: &BuiltinFunc,
        this: This,
        args: Args,
    ) -> RuntimeResult {
        log::trace!("BEGIN: call {} with this: {}", func.name(), this_to_str(&this));
        log::trace!("ARGS: {}", args_to_str(&args));
        let args = self.check_call_args(func, &this, args)?;
        self.push_call_frame(0, None)?;
        let result = (func.func)(this, args, self);
        match result {
            Ok(return_val) => {
                self.push_return_val(return_val);
                self.pop_call_frame()?;
                Ok(())
            }
            Err(err) => {
                self.reset();
                Err(err)
            }
        }
    }

    pub fn call_func(&mut self, func: &Func, num_args: usize) -> RuntimeResult {
        log::trace!("BEGIN: call {}\n{}", func.name(), self.format_stack());
        let args_offset = self.check_func_args(func, num_args)?;
        self.push_call_frame(args_offset, None)?;
        match self.execute(&func.code) {
            Ok(_) => {
                self.pop_call_frame()?;
                Ok(())
            }
            Err(err) => {
                self.reset();
                Err(err)
            }
        }
    }

    pub fn call_func_direct(&mut self, func: &Func, args: Args) -> RuntimeResult {
        log::trace!(
            "BEGIN: call {} directly with args: {}",
            func.name(),
            args_to_str(&args)
        );
        let args = self.check_call_args(func, &None, args)?;
        self.enter_scope(0);
        self.push_call_frame(0, None)?;
        for arg in args {
            self.push_temp(arg);
        }
        match self.execute(&func.code) {
            Ok(_) => {
                self.exit_scope();
                self.pop_call_frame()?;
                Ok(())
            }
            Err(err) => {
                self.reset();
                Err(err)
            }
        }
    }

    pub fn call_closure(
        &mut self,
        closure_ref: ObjectRef,
        num_args: usize,
    ) -> RuntimeResult {
        let closure = closure_ref.read().unwrap();
        let closure = closure.down_to_closure().unwrap();
        log::trace!("BEGIN: call closure {}", closure.name());
        let func = closure.func.read().unwrap();
        let func = func.down_to_func().unwrap();
        let args_offset = self.check_func_args(func, num_args)?;
        self.push_call_frame(args_offset, Some(closure_ref.clone()))?;
        match self.execute(&func.code) {
            Ok(_) => {
                self.pop_call_frame()?;
                Ok(())
            }
            Err(err) => {
                self.reset();
                Err(err)
            }
        }
    }

    /// Check args for function. These N args are sitting at the top of
    /// the stack. This also handles mapping vars into a tuple. The
    /// return value is the "args offset" which is used to adjust the
    /// call frame pointer to account for args.
    fn check_func_args(
        &mut self,
        func: &Func,
        num_args: usize,
    ) -> Result<usize, RuntimeErr> {
        let name = func.name();
        let arity = func.arity();
        if let Some(var_args_index) = func.var_args_index() {
            if num_args < arity {
                self.check_arity(name, arity, num_args, &None)?;
            }
            let objects = self.pop_n_obj(num_args - var_args_index)?;
            let tuple = create::new_tuple(objects);
            self.push_temp(tuple);
            Ok(arity + 1)
        } else {
            self.check_arity(name, arity, num_args, &None)?;
            Ok(num_args)
        }
    }

    /// Check call args to ensure they're valid. This ensures the
    /// function was called with the required number args and also takes
    /// care of mapping var args into a tuple in the last position.
    fn check_call_args(
        &self,
        func: &dyn FuncTrait,
        this: &This,
        args: Args,
    ) -> Result<Args, RuntimeErr> {
        let name = func.name();
        let arity = func.arity();
        if let Some(var_args_index) = func.var_args_index() {
            let n_args = args.iter().take(var_args_index).len();
            self.check_arity(name, arity, n_args, this)?;
            let mut args = args.clone();
            let var_args_items = args.split_off(var_args_index);
            let var_args = create::new_tuple(var_args_items);
            args.push(var_args);
            Ok(args)
        } else {
            self.check_arity(name, arity, args.len(), this)?;
            Ok(args)
        }
    }

    fn check_arity(
        &self,
        name: &str,
        arity: usize,
        n_args: usize,
        this: &This,
    ) -> RuntimeResult {
        if n_args != arity {
            let ess = if arity == 1 { "" } else { "s" };
            let msg = format!(
                "{}{}() expected {arity} arg{ess}; got {n_args}",
                this.clone().map_or_else(
                    || "".to_owned(),
                    |this| {
                        let this = this.read().unwrap();
                        let class = this.class();
                        let class = class.read().unwrap();
                        format!("{}.", class.name())
                    }
                ),
                name
            );
            return Err(RuntimeErr::type_err(msg));
        }
        Ok(())
    }

    // Call Stack ------------------------------------------------------

    fn push_call_frame(
        &mut self,
        args_offset: usize,
        closure: Option<ObjectRef>,
    ) -> RuntimeResult {
        if self.call_stack_size == self.max_call_depth {
            self.reset();
            return Err(RuntimeErr::recursion_depth_exceeded(self.max_call_depth));
        }
        let stack_position = self.value_stack.len() - args_offset;
        let frame = CallFrame::new(stack_position, closure);
        self.call_stack.push(frame);
        self.call_stack_size += 1;
        self.call_frame_pointer = stack_position;
        Ok(())
    }

    fn pop_call_frame(&mut self) -> Result<CallFrame, RuntimeErr> {
        match self.call_stack.pop() {
            Some(frame) => {
                let size = self.call_stack_size - 1;
                self.call_stack_size = size;
                self.call_frame_pointer =
                    if size == 0 { 0 } else { self.call_stack[size - 1].stack_pointer };
                Ok(frame)
            }
            None => Err(RuntimeErr::empty_call_stack()),
        }
    }

    fn current_call_frame(&self) -> Result<&CallFrame, RuntimeErr> {
        if let Some(frame) = self.call_stack.peek() {
            Ok(frame)
        } else {
            Err(RuntimeErr::empty_call_stack())
        }
    }

    // Scopes ----------------------------------------------------------

    fn enter_scope(&mut self, args_offset: usize) {
        self.scope_stack.push(self.value_stack.len() - args_offset);
        self.ctx.enter_scope();
    }

    /// When exiting a scope, we first save the top of the stack (which
    /// is the "return value" of the scope), remove all stack values
    /// added in the scope, including locals, and finally push the
    /// scope's "return value" back onto the stack. Finally, the scope's
    /// namespace is then cleared and removed.
    fn exit_scope(&mut self) {
        log::trace!("STACK BEFORE EXIT SCOPE:\n{}", self.format_stack());
        let return_val = self.pop_obj();
        if let Some(pointer) = self.scope_stack.pop() {
            self.value_stack.truncate(pointer);
        } else {
            panic!("Scope stack unexpectedly empty when exiting scope");
        };
        // Ensure the scope left a value on the stack.
        if let Ok(obj) = return_val {
            self.push_return_val(obj);
        } else {
            panic!("Value stack unexpectedly empty when exiting scope");
        }
        // Clear scope namespaces.
        self.ctx.exit_scope();
        log::trace!("STACK AFTER EXIT SCOPE:\n{}", self.format_stack());
    }

    /// This is a convenience for jumping out multiple scopes when
    /// jumping.
    fn exit_scopes(&mut self, count: usize) {
        if count > 0 {
            for _ in 0..count {
                self.exit_scope();
            }
        }
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

    /// This is used to create new local slots on the stack. These slots
    /// remain on the stack until the current scope is exited.
    ///
    /// NOTE: This is only used when calling a user function to store
    ///       its args as locals.
    fn push_local(&mut self, obj: ObjectRef, index: usize) {
        self.push(ValueStackKind::Local(obj, index));
    }

    /// This is used when loading (copying) a local from its slot to
    /// TOS. These "temporary locals" are cleaned up normally.
    fn push_temp_local(&mut self, obj: ObjectRef, index: usize) {
        self.push(ValueStackKind::TempLocal(obj, index));
    }

    /// Replace value stack item at index.
    fn replace(&mut self, index: usize, kind: ValueStackKind) -> RuntimeResult {
        if index < self.value_stack.len() {
            self.value_stack.set_at(index, kind);
            Ok(())
        } else {
            Err(RuntimeErr::stack_index_out_of_bounds(index))
        }
    }

    /// Store object into local slot.
    fn store_local(&mut self, obj: ObjectRef, index: usize) -> RuntimeResult {
        let frame_index = self.call_frame_pointer + index;
        if frame_index < self.value_stack.len() {
            self.replace(frame_index, ValueStackKind::Local(obj, index))
        } else {
            Err(RuntimeErr::frame_index_out_of_bounds(frame_index))
        }
    }

    /// Load (copy) object from local slot to TOS.
    fn load_local(&mut self, index: usize) -> RuntimeResult {
        let frame_index = self.call_frame_pointer + index;
        if let Some(kind) = self.value_stack.peek_at(frame_index) {
            if let ValueStackKind::Local(obj, index) = kind {
                self.push_temp_local(obj.clone(), *index);
                Ok(())
            } else {
                panic!("Expected local; got {kind:?}");
            }
        } else {
            Err(RuntimeErr::frame_index_out_of_bounds(frame_index))
        }
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
            None => Err(RuntimeErr::empty_stack()),
        }
    }

    pub fn pop_obj(&mut self) -> PopObjResult {
        let kind = self.pop()?;
        Ok(self.get_obj(&kind))
    }

    fn pop_n(&mut self, n: usize) -> PopNResult {
        match self.value_stack.pop_n(n) {
            Some(kinds) => Ok(kinds),
            None => Err(RuntimeErr::not_enough_values_on_stack(n)),
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
            None => Err(RuntimeErr::empty_stack()),
        }
    }

    pub fn peek_obj(&mut self) -> PeekObjResult {
        let kind = self.peek()?;
        Ok(self.get_obj(kind))
    }

    fn peek_at(&self, index: usize) -> PeekResult {
        match self.value_stack.peek_at(index) {
            Some(kind) => Ok(kind),
            None => Err(RuntimeErr::stack_index_out_of_bounds(index)),
        }
    }

    fn peek_at_obj(&self, index: usize) -> PeekObjResult {
        let kind = self.peek_at(index)?;
        Ok(self.get_obj(kind))
    }

    fn get_obj(&self, kind: &ValueStackKind) -> ObjectRef {
        use ValueStackKind::*;
        match kind {
            GlobalConstant(obj, ..) => obj.clone(),
            Constant(obj, ..) => obj.clone(),
            Var(obj, ..) => obj.clone(),
            Local(obj, ..) => obj.clone(),
            TempLocal(obj, ..) => obj.clone(),
            Temp(obj) => obj.clone(),
            ReturnVal(obj) => obj.clone(),
        }
    }

    // Utilities -------------------------------------------------------

    /// Show the contents of the stack (top first).
    pub fn display_stack(&self) {
        eprintln!("{}", self.format_stack());
    }

    pub fn format_stack(&self) -> String {
        use ValueStackKind::*;
        if self.value_stack.is_empty() {
            return "[EMPTY]".to_owned();
        }
        let mut items = vec![];
        for (i, kind) in self.value_stack.iter().enumerate() {
            let kind_marker = match kind {
                GlobalConstant(..) => "G",
                Constant(..) => "C",
                Var(..) => "V",
                Local(..) => "L",
                Temp(..) => "T",
                TempLocal(..) => "TL",
                ReturnVal(..) => "R",
            };
            let obj = self.get_obj(kind);
            let obj = &*obj.read().unwrap();
            let top_marker = if i == 0 { "TOS" } else { "     " };
            let frame_marker = if i == self.call_frame_pointer { "<----" } else { "" };
            let string =
                format!("{top_marker: <8}{kind_marker: <4}{obj:?}{frame_marker: >12}");
            items.push(string)
        }
        items.join("\n")
    }
}
