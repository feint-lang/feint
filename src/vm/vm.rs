//! The FeInt virtual machine. When it's created, it's initialized and
//! then, implicitly, goes idle until it's passed some instructions to
//! execute. After instructions are executed, it goes back into idle
//! mode.
use std::cmp;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use ctrlc;
use num_traits::ToPrimitive;

use crate::modules;
use crate::source::Location;
use crate::types::{
    new, Args, BuiltinFunc, Func, FuncTrait, Module, ObjectRef, ThisOpt,
};
use crate::util::{
    BinaryOperator, CompareOperator, InplaceOperator, Stack, UnaryCompareOperator,
    UnaryOperator,
};

use super::code::Code;
use super::context::RuntimeContext;
use super::globals;
use super::inst::{Inst, PrintFlags};
use super::result::{
    CallDepth, PeekObjResult, PeekResult, PopNObjResult, PopNResult, PopObjResult,
    PopResult, RuntimeErr, RuntimeObjResult, RuntimeResult, VMExeResult, VMState,
    ValueStackKind,
};

pub const DEFAULT_MAX_CALL_DEPTH: CallDepth =
    if cfg!(debug_assertions) { 256 } else { 1024 };

struct CallFrame {
    stack_pointer: usize,
    this_opt: ThisOpt,
    closure: Option<ObjectRef>,
}

impl CallFrame {
    pub fn new(
        stack_pointer: usize,
        this_opt: ThisOpt,
        closure: Option<ObjectRef>,
    ) -> Self {
        Self { stack_pointer, this_opt, closure }
    }

    pub fn get_captured(&self, name: &str) -> RuntimeObjResult {
        if let Some(closure) = &self.closure {
            let closure = closure.read().unwrap();
            let closure = closure.down_to_closure().unwrap();
            if let Some(obj) = closure.captured().get(name) {
                return Ok(obj.clone());
            }
        }
        Err(RuntimeErr::captured_var_not_found(name))
    }
}

pub struct VM {
    pub(crate) ctx: RuntimeContext,
    pub(crate) state: VMState,
    global_constants: Vec<ObjectRef>,
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
            state: VMState::Idle(None),
            global_constants: globals::get_global_constants(),
            scope_stack: Stack::with_capacity(max_call_depth),
            value_stack: Stack::with_capacity(max_call_depth * 8),
            call_stack: Stack::with_capacity(max_call_depth),
            max_call_depth,
            loc: (Location::default(), Location::default()),
            handle_sigint: sigint_flag.load(Ordering::Relaxed),
            sigint_flag,
        }
    }

    pub fn execute_module(&mut self, module: &mut Module, start: usize) -> VMExeResult {
        self.execute_code(Some(module), module.code(), start)
    }

    pub fn execute_func(&mut self, func: &Func, start: usize) -> VMExeResult {
        self.execute_code(None, func.code(), start)
    }

    /// Execute the given code object's instructions and return the VM's
    /// state.
    ///
    /// When a HALT instruction is encountered, the VM's state will be
    /// cleared and an `Exit` error will be returned.
    ///
    /// If a HALT instruction is *not* encountered, the VM will go
    /// "idle"--it will maintain its internal state and await further
    /// instructions.
    pub fn execute_code(
        &mut self,
        module: Option<&Module>,
        code: &Code,
        start: usize,
    ) -> VMExeResult {
        use Inst::*;

        self.set_running();

        let handle_sigint = self.handle_sigint;
        let sigint_flag = self.sigint_flag.clone();
        let mut sigint_counter = 0u32;

        let mut ip = start;
        let mut jump_ip = None;

        let len_chunk = code.len_chunk();

        match start.cmp(&len_chunk) {
            cmp::Ordering::Equal => {
                self.set_idle(None);
                return Ok(());
            }
            cmp::Ordering::Greater => panic!("Code start index out of bounds"),
            _ => (),
        }

        loop {
            match &code[ip] {
                NoOp => {
                    // do nothing
                }
                Pop => {
                    self.pop()?;
                }
                // Well-known global constants
                LoadNil => {
                    self.push_global_const(globals::NIL_INDEX)?;
                }
                LoadTrue => {
                    self.push_global_const(globals::TRUE_INDEX)?;
                }
                LoadFalse => {
                    self.push_global_const(globals::FALSE_INDEX)?;
                }
                LoadAlways => {
                    self.push_global_const(globals::ALWAYS_INDEX)?;
                }
                LoadEmptyStr => {
                    self.push_global_const(globals::EMPTY_STR_INDEX)?;
                }
                LoadNewline => {
                    self.push_global_const(globals::NEWLINE_INDEX)?;
                }
                LoadEmptyTuple => {
                    self.push_global_const(globals::EMPTY_TUPLE_INDEX)?;
                }
                // Other global constants (shared ints)
                LoadGlobalConst(index) => {
                    self.push_global_const(*index)?;
                }
                // Scopes
                ScopeStart => {
                    self.enter_scope();
                }
                ScopeEnd => {
                    self.exit_scope();
                }
                StatementStart(start, end) => {
                    self.loc = (*start, *end);
                }
                LoadConst(index) => {
                    let obj = code.get_const(*index)?.clone();
                    self.push(ValueStackKind::Constant(obj, *index));
                }
                // Modules
                LoadModule(name) => {
                    let module = modules::get_module(name.as_str())?;
                    self.push_temp(module);
                }
                // Vars
                DeclareVar(name) => {
                    if self.ctx.get_var_in_current_ns(name).is_err() {
                        self.ctx.declare_var(name.as_str());
                    }
                }
                AssignVar(name) => {
                    let obj = self.pop_obj()?;
                    let depth = self.ctx.assign_var(name, obj)?;
                    self.push_var(depth, name.clone())?;
                }
                LoadVar(name) => {
                    if let Ok(depth) = self.ctx.get_var_depth(name.as_str(), None) {
                        self.push_var(depth, name.clone())?;
                    } else {
                        self.push_temp_from_module_or_builtins(name, module);
                    }
                }
                LoadOuterVar(name) => {
                    if let Ok(depth) = self.ctx.get_outer_var_depth(name.as_str()) {
                        self.push_var(depth, name.clone())?;
                    } else {
                        self.push_temp_from_module_or_builtins(name, module);
                    }
                }
                AssignCell(name) => {
                    // Store TOS value into cell. This is similar to
                    // AssignVar except that it wraps the TOS value in
                    // a cell before storing it as var.
                    let value = self.pop_obj()?;
                    // Get the var, which might not already be a cell.
                    let var_ref = self.ctx.get_var(name.as_str())?;
                    let mut var = var_ref.write().unwrap();
                    let depth = if let Some(cell) = var.down_to_cell_mut() {
                        // Wrap TOS in existing cell.
                        cell.set_value(value.clone());
                        self.ctx.assign_var(name, var_ref.clone())?
                    } else {
                        // Create new cell to wrap TOS in.
                        assert!(var.is_nil());
                        let cell_ref = new::cell_with_value(value.clone());
                        self.ctx.assign_var(name, cell_ref)?
                    };
                    // Push cell *value* to TOS.
                    self.push(ValueStackKind::CellVar(value, depth, name.to_owned()));
                }
                LoadCell(name) => {
                    // Load cell value onto TOS. This is similar to
                    // LoadVar except that it unwraps the value from the
                    // retrieved cell.
                    log::trace!("LOAD CELL: {name}");
                    let depth = self.ctx.get_var_depth(name.as_str(), None)?;
                    let cell = self.ctx.get_var_at_depth(depth, name.as_str())?;
                    let cell = cell.read().unwrap();
                    let cell =
                        cell.down_to_cell().expect("Expected cell: {name} @ {ip}");
                    let value = cell.value();
                    // Push cell *value* to TOS.
                    self.push(ValueStackKind::CellVar(value, depth, name.to_owned()));
                }
                LoadCaptured(name) => {
                    // This is similar to LoadCell except that it loads
                    // a cell from the current closure, unwraps its
                    // value, and loads it to TOS as a temporary.
                    let frame = self.current_call_frame()?;
                    if frame.closure.is_some() {
                        let cell = frame.get_captured(name)?;
                        let cell = cell.read().unwrap();
                        let cell =
                            cell.down_to_cell().expect("Expected cell: {name} @ {ip}");
                        let value = cell.value();
                        self.push_temp(value);
                    } else {
                        panic!("Expected closure");
                    }
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
                Call(num_args) => {
                    log::trace!("STACK before call:\n{}", self.format_stack());
                    let callable = self.pop_obj()?;
                    let args = self.pop_n_obj(*num_args)?;
                    log::trace!("STACK before call:\n{}", self.format_stack());
                    self.call(callable, args)?;
                }
                Return => {
                    // RETURN doesn't do anything in and of itself. It's
                    // a marker for the end of a function and a jump
                    // target for explicit returns.
                }
                // Object construction
                MakeString(n) => {
                    let objects = self.pop_n_obj(*n)?;
                    let mut string = String::with_capacity(32);
                    for obj in objects {
                        let obj = obj.read().unwrap();
                        string.push_str(obj.to_string().as_str());
                    }
                    let string_obj = new::str(string);
                    self.push_temp(string_obj);
                }
                MakeTuple(n) => {
                    let objects = self.pop_n_obj(*n)?;
                    let tuple = new::tuple(objects);
                    self.push_temp(tuple);
                }
                MakeList(n) => {
                    let objects = self.pop_n_obj(*n)?;
                    let list = new::list(objects);
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
                    let map = new::map(entries);
                    self.push_temp(map);
                }
                CaptureSet(names) => {
                    let mut entries = vec![];
                    for name in names.iter() {
                        log::trace!("GETTING CAPTURED: {name}");
                        if let Ok(var_ref) = self.ctx.get_var(name) {
                            // Capture cell already exists.
                            let var = var_ref.read().unwrap();
                            if var.is_cell() {
                                entries.push((name.to_owned(), var_ref.clone()));
                            } else {
                                assert!(var.is_nil());
                                entries.push((name.to_owned(), new::cell()));
                            }
                        } else {
                            // Capture cell does not exist.
                            if let Some(frame) = self.call_stack.peek() {
                                if let Some(closure) = &frame.closure {
                                    log::trace!("CAPTURING OUTER");
                                    let closure = closure.read().unwrap();
                                    let closure = closure.down_to_closure().unwrap();
                                    let result = closure
                                        .captured()
                                        .iter()
                                        .find(|(n, _)| *n == name);
                                    if let Some((name, cell)) = result {
                                        entries.push((name.to_owned(), cell.clone()));
                                        log::trace!(
                                            "CAPTURED FROM OUTER: {name} = {cell:?}"
                                        );
                                    }
                                }
                            }
                        }
                    }
                    self.push_temp(new::map(entries));
                }
                MakeFunc => {
                    let capture_set = self.pop_obj()?;
                    let capture_set = capture_set.read().unwrap();
                    let capture_set = capture_set.down_to_map().unwrap();
                    if !capture_set.is_empty() {
                        let func_ref = self.pop_obj()?;
                        let func_obj = func_ref.read().unwrap();
                        let func = func_obj.down_to_func().unwrap();
                        let mut captured = capture_set.to_hash_map();

                        // XXX: This gets around a chicken-and-egg
                        //      problem when the closure references
                        //      itself.
                        let func_captured = captured.contains_key(func.name());
                        if func_captured && ip + 1 < code.len_chunk() {
                            if let AssignCell(_) = &code[ip + 1] {
                                let closure_cell =
                                    new::cell_with_value(func_ref.clone());
                                self.ctx
                                    .assign_var(func.name(), closure_cell.clone())?;
                                captured.insert(func.name().to_owned(), closure_cell);
                            }
                        }

                        self.push_temp(new::closure(func_ref.clone(), captured));
                    }
                }
                // VM control
                Halt(return_code) => {
                    return self.halt(*return_code);
                }
                HaltTop => {
                    let obj = self.pop_obj()?;
                    let obj = obj.read().unwrap();
                    let return_code = match obj.get_int_val() {
                        Some(int) => int.to_u8().unwrap_or(0),
                        None => 0,
                    };
                    return self.halt(return_code);
                }
                // Placeholders
                Placeholder(addr, inst, message) => {
                    eprintln!(
                        "Placeholder at {addr} was not updated: {inst:?}\n{message}"
                    );
                    return self.halt(255);
                }
                FreeVarPlaceholder(addr, name) => {
                    eprintln!("Var placeholder at {addr} was not updated: {name}");
                    return self.halt(255);
                }
                BreakPlaceholder(addr, _) => {
                    eprintln!("Break placeholder at {addr} was not updated");
                    return self.halt(255);
                }
                ContinuePlaceholder(addr, _) => {
                    eprintln!("Continue placeholder at {addr} was not updated");
                    return self.halt(255);
                }
                ReturnPlaceholder(addr, _) => {
                    eprintln!("Return placeholder at {addr} was not updated");
                    return self.halt(255);
                }
                // Miscellaneous
                Print(flags) => {
                    self.handle_print(flags)?;
                }
                DisplayStack(message) => {
                    eprintln!("\nSTACK: {message}\n");
                    self.display_stack();
                    eprintln!();
                }
            }

            if handle_sigint {
                sigint_counter += 1;
                // TODO: Maybe use a different value and/or make it
                //       configurable.
                if sigint_counter == 1024 {
                    if sigint_flag.load(Ordering::Relaxed) {
                        self.handle_sigint();
                        self.set_idle(None);
                        break Ok(());
                    }
                    sigint_counter = 0;
                }
            }

            if let Some(new_ip) = jump_ip {
                ip = new_ip;
                jump_ip = None;
            } else {
                ip += 1;
                if ip == len_chunk {
                    let top = self.peek_obj().map_or_else(|_| None, Some);
                    self.set_idle(top.clone());
                    break Ok(());
                }
            }
        }
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

    // State -----------------------------------------------------------

    fn set_running(&mut self) {
        self.state = VMState::Running;
    }

    fn set_idle(&mut self, obj: Option<ObjectRef>) {
        self.state = VMState::Idle(obj);
    }

    fn halt(&mut self, exit_code: u8) -> VMExeResult {
        self.reset();
        self.state = VMState::Halted(exit_code);
        Err(RuntimeErr::exit(exit_code))
    }

    /// Completely reset internal state.
    fn reset(&mut self) {
        self.scope_stack.truncate(0);
        self.value_stack.truncate(0);
        self.call_stack.truncate(0);
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
        self.push_temp(new::bool(result));
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
                    let mut result = a.get_attr(name, a_ref.clone());

                    // If name isn't an attr and LHS is a sequence, look
                    // up `name` and use its value as an index, if
                    // possible. If this fails--if `name` isn't defined
                    // or isn't an index--the original attr err will be
                    // returned.
                    if result.read().unwrap().is_err() && (a.is_seq()) {
                        let i = self.ctx.get_var(name);
                        if let Ok(i) = i {
                            let i = i.read().unwrap();
                            if let Some(i) = i.get_usize_val() {
                                result = a.get_item(i, a_ref.clone());
                            }
                        }
                    }

                    result
                } else if let Some(index) = b.get_usize_val() {
                    a.get_item(index, a_ref.clone())
                } else {
                    // XXX: This can happen for a construct like `1.()`,
                    //      but that should probably be a syntax error
                    //      that's caught early.
                    new::attr_err(
                        format!("Not an attribute name or index: {b:?}"),
                        a_ref.clone(),
                    )
                };

                let obj = obj_ref.read().unwrap();
                if obj.is_builtin_func() || obj.is_func() || obj.is_closure() {
                    // If `b` in `a.b` is a function, bind `b` to `a`.

                    // TODO: Check whether `a` is a type or an instance.

                    new::bound_func(obj_ref.clone(), a_ref.clone())
                } else if let Some(prop) = obj.down_to_prop() {
                    // If `b` in `a.b` is a property, bind `b`'s getter
                    // to `a` then call the bound getter.

                    // TODO: Check whether `a` is a type or an instance
                    //       and return the property itself when `a` is
                    //       a type.

                    let func = new::bound_func(prop.getter(), a_ref.clone());
                    if a.is_type_object() {
                        func
                    } else {
                        return self.call(func, vec![]);
                    }
                } else {
                    drop(obj);
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
            IsTypeEqual => a.is_type_equal(b),
            IsNotTypeEqual => !a.is_type_equal(b),
            IsEqual => a.is_equal(b),
            NotEqual => !a.is_equal(b),
            And => a.and(b)?,
            Or => a.or(b)?,
            LessThan => a.less_than(b)?,
            LessThanOrEqual => a.less_than(b)? || a.is_equal(b),
            GreaterThan => a.greater_than(b)?,
            GreaterThanOrEqual => a.greater_than(b)? || a.is_equal(b),
        };
        self.push_temp(new::bool(result));
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
            InplaceOperator::Mul => a.mul(b)?,
            InplaceOperator::Div => a.div(b)?,
            InplaceOperator::Add => a.add(b)?,
            InplaceOperator::Sub => a.sub(b)?,
        };
        if let ValueStackKind::Var(_, depth, name) = a_kind {
            self.ctx.assign_var_at_depth(depth, name.as_str(), result.clone())?;
            self.push_temp(result);
        } else if let ValueStackKind::CellVar(_, depth, name) = a_kind {
            let cell = self.ctx.get_var_at_depth(depth, name.as_str())?;
            let mut cell = cell.write().unwrap();
            let cell = cell.down_to_cell_mut().expect("Expected cell");
            cell.set_value(result.clone());
            self.push_temp(result);
        } else {
            return Err(RuntimeErr::expected_var(format!("Binary op: {}", op)));
        }
        Ok(())
    }

    fn handle_print(&mut self, flags: &PrintFlags) -> RuntimeResult {
        if let Ok(obj) = self.pop_obj() {
            let obj = obj.read().unwrap();
            if flags.contains(PrintFlags::NO_NIL) && obj.is_nil() {
                // do nothing
            } else if flags.contains(PrintFlags::ERR) {
                if flags.contains(PrintFlags::REPR) {
                    eprint!("{:?}", &*obj);
                } else {
                    eprint!("{obj}");
                }
                if flags.contains(PrintFlags::NL) {
                    eprintln!();
                }
            } else {
                if flags.contains(PrintFlags::REPR) {
                    print!("{:?}", &*obj);
                } else {
                    print!("{obj}");
                }
                if flags.contains(PrintFlags::NL) {
                    println!();
                }
            }
            Ok(())
        } else {
            Err(RuntimeErr::empty_stack())
        }
    }

    // Call Stack ------------------------------------------------------

    // NOTE: Pushing a call frame is similar to entering a scope.
    fn push_call_frame(
        &mut self,
        this_opt: ThisOpt,
        closure: Option<ObjectRef>,
    ) -> RuntimeResult {
        if self.call_stack.len() == self.max_call_depth {
            self.reset();
            return Err(RuntimeErr::recursion_depth_exceeded(self.max_call_depth));
        }
        self.ctx.enter_scope();
        let stack_pointer = self.value_stack.len();
        let frame = CallFrame::new(stack_pointer, this_opt, closure);
        self.call_stack.push(frame);
        Ok(())
    }

    // NOTE: Popping a call frame is very similar to exiting a scope.
    fn pop_call_frame(&mut self) -> RuntimeResult {
        let return_val = self.pop_obj();
        if let Some(frame) = self.call_stack.pop() {
            self.value_stack.truncate(frame.stack_pointer);
        } else {
            panic!("Call stack unexpectedly empty");
        }
        // Ensure the frame left a value on the stack.
        if let Ok(obj) = return_val {
            self.push_return_val(obj.clone());
        } else {
            panic!("Value stack unexpectedly empty when exiting scope");
        }
        self.ctx.exit_scope();
        Ok(())
    }

    fn current_call_frame(&self) -> Result<&CallFrame, RuntimeErr> {
        if let Some(frame) = self.call_stack.peek() {
            Ok(frame)
        } else {
            Err(RuntimeErr::empty_call_stack())
        }
    }

    #[inline]
    fn call_frame_pointer(&self) -> usize {
        if self.call_stack.is_empty() {
            0
        } else {
            self.call_stack[self.call_stack.len() - 1].stack_pointer
        }
    }

    /// Look up call chain for `this`.
    fn find_this(&self) -> ObjectRef {
        for frame in self.call_stack.iter() {
            if let Some(this) = &frame.this_opt {
                return this.clone();
            }
        }
        new::nil()
    }

    // Function calls --------------------------------------------------

    pub fn call(&mut self, callable_ref: ObjectRef, args: Args) -> RuntimeResult {
        let callable = callable_ref.read().unwrap();
        if let Some(func) = callable.down_to_builtin_func() {
            log::trace!("CALL builtin func {}", func.name());
            self.call_builtin_func(func, None, args)
        } else if let Some(func) = callable.down_to_func() {
            log::trace!("CALL func {}", func.name());
            self.call_func(func, None, args, None)
        } else if callable.is_closure() {
            log::trace!("CALL closure");
            self.call_closure(callable_ref.clone(), None, args)
        } else if let Some(bound_func) = callable.down_to_bound_func() {
            let func_ref = bound_func.func();
            let func_obj = func_ref.read().unwrap();
            let this_opt = Some(bound_func.this());
            if let Some(func) = func_obj.down_to_builtin_func() {
                log::trace!(
                    "CALL bound builtin func {} with this: {}",
                    func.name(),
                    bound_func.this().read().unwrap()
                );
                if let Some(expected_type) = func.this_type() {
                    let expected_type = &*expected_type.read().unwrap();
                    let this = bound_func.this();
                    let this = this.read().unwrap();
                    let this_type = this.type_obj();
                    let this_type = this_type.read().unwrap();
                    // class method || instance method
                    // XXX: Not sure this is the best way to distinguish
                    //      between class vs instance methods
                    if !(this.is(expected_type) || this_type.is(expected_type)) {
                        panic!("Expected this type {expected_type}; got {this_type}");
                    }
                }
                self.call_builtin_func(func, this_opt, args)
            } else if let Some(func) = func_obj.down_to_func() {
                log::trace!(
                    "CALL bound func {} with this: {}",
                    func.name(),
                    bound_func.this().read().unwrap()
                );
                self.call_func(func, this_opt, args, None)
            } else if callable.is_closure() {
                log::trace!(
                    "CALL bound closure with this: {}",
                    bound_func.this().read().unwrap()
                );
                self.call_closure(func_ref.clone(), this_opt, args)
            } else {
                Err(func_obj.not_callable())
            }
        } else {
            Err(callable.not_callable())
        }
    }

    fn call_builtin_func(
        &mut self,
        func: &BuiltinFunc,
        this_opt: ThisOpt,
        args: Args,
    ) -> RuntimeResult {
        let args = self.check_call_args(func, &this_opt, args)?;
        self.push_call_frame(this_opt.clone(), None)?;
        let result = (func.func())(self.find_this(), args, self);
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

    pub fn call_func(
        &mut self,
        func: &Func,
        this_opt: ThisOpt,
        args: Args,
        closure: Option<ObjectRef>,
    ) -> RuntimeResult {
        let args = self.check_call_args(func, &None, args)?;
        self.push_call_frame(this_opt, closure)?;
        self.ctx.declare_and_assign_var("this", self.find_this())?;
        // XXX: All args are created as cells, which allows them to be
        //      captured without having to track whether they were in
        //      fact captured. This isn't a great solution--it would be
        //      better to track which params are captured. See related
        //      note in push_var().
        for (name, arg) in func.arg_names().iter().zip(args) {
            let cell = new::cell_with_value(arg);
            self.ctx.declare_and_assign_var(name, cell)?;
        }
        match self.execute_func(func, 0) {
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

    pub fn call_closure(
        &mut self,
        closure_ref: ObjectRef,
        this_opt: ThisOpt,
        args: Args,
    ) -> RuntimeResult {
        let closure = closure_ref.read().unwrap();
        let closure = closure.down_to_closure().unwrap();
        let func = closure.func();
        let func = func.read().unwrap();
        let func = func.down_to_func().unwrap();
        self.call_func(func, this_opt, args, Some(closure_ref.clone()))
    }

    /// Check call args to ensure they're valid. This ensures the
    /// function was called with the required number args and also takes
    /// care of mapping var args into a tuple in the last position.
    fn check_call_args(
        &self,
        func: &dyn FuncTrait,
        this_opt: &ThisOpt,
        args: Args,
    ) -> Result<Args, RuntimeErr> {
        let name = func.name();
        let arity = func.arity();
        if let Some(var_args_index) = func.var_args_index() {
            let n_args = args.iter().take(var_args_index).len();
            self.check_arity(name, arity, n_args, this_opt)?;
            let mut args = args.clone();
            let var_args_items = args.split_off(var_args_index);
            let var_args = new::tuple(var_args_items);
            args.push(var_args);
            Ok(args)
        } else {
            self.check_arity(name, arity, args.len(), this_opt)?;
            Ok(args)
        }
    }

    fn check_arity(
        &self,
        name: &str,
        arity: usize,
        num_args: usize,
        this_opt: &ThisOpt,
    ) -> RuntimeResult {
        if num_args != arity {
            let ess = if arity == 1 { "" } else { "s" };
            let msg = format!(
                "{}{}() expected {arity} arg{ess}; got {num_args}",
                this_opt.clone().map_or_else(
                    || "".to_owned(),
                    |this_ref| {
                        let this_obj = this_ref.read().unwrap();
                        format!("{}.", this_obj.class().read().unwrap().full_name())
                    }
                ),
                name
            );
            return Err(RuntimeErr::type_err(msg));
        }
        Ok(())
    }

    // Scopes ----------------------------------------------------------

    fn enter_scope(&mut self) {
        self.ctx.enter_scope();
        self.scope_stack.push(self.value_stack.len());
    }

    /// When exiting a scope, we first save the top of the stack (which
    /// is the "return value" of the scope), remove all stack values
    /// added in the scope, including locals, and finally push the
    /// scope's "return value" back onto the stack. Finally, the scope's
    /// namespace is then cleared and removed.
    fn exit_scope(&mut self) {
        let return_val = self.pop_obj();
        if let Some(pointer) = self.scope_stack.pop() {
            self.value_stack.truncate(pointer);
        } else {
            panic!("Scope stack unexpectedly empty when exiting scope");
        };
        // Ensure the scope left a value on the stack.
        if let Ok(obj) = return_val {
            self.push_return_val(obj.clone());
        } else {
            panic!("Value stack unexpectedly empty when exiting scope");
        }
        self.ctx.exit_scope();
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
        if let Some(obj) = self.global_constants.get(index) {
            self.push(ValueStackKind::GlobalConstant(obj.clone(), index));
            Ok(())
        } else {
            Err(RuntimeErr::constant_not_found(index))
        }
    }

    fn push_var(&mut self, depth: usize, name: String) -> RuntimeResult {
        let obj_ref = self.ctx.get_var_at_depth(depth, name.as_str())?;
        // XXX: This is a workaround for function args being created
        //      as cells.
        let obj = obj_ref.read().unwrap();
        if let Some(cell) = obj.down_to_cell() {
            let value = cell.value();
            self.push(ValueStackKind::CellVar(value, depth, name));
        } else {
            self.push(ValueStackKind::Var(obj_ref.clone(), depth, name));
        }
        Ok(())
    }

    /// This is the fallback when loading a var fails because `name`
    /// isn't found. First, try getting the var from the specified
    /// `module`'s globals (if a module was specified). Otherwise, fall
    /// back to the builtins.
    fn push_temp_from_module_or_builtins(
        &mut self,
        name: &str,
        module: Option<&Module>,
    ) {
        if let Some(module) = module {
            if let Some(obj) = module.get_global(name) {
                self.push_temp(obj);
            } else {
                let obj = self.ctx.get_builtin(name);
                self.push_temp(obj);
            }
        } else {
            let obj = self.ctx.get_builtin(name);
            self.push_temp(obj);
        }
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
        if n == 0 {
            return Ok(vec![]);
        }
        match self.value_stack.pop_n(n) {
            Some(kinds) => Ok(kinds),
            None => Err(RuntimeErr::not_enough_values_on_stack(n)),
        }
    }

    fn pop_n_obj(&mut self, n: usize) -> PopNObjResult {
        if n == 0 {
            return Ok(vec![]);
        }
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

    fn get_obj(&self, kind: &ValueStackKind) -> ObjectRef {
        use ValueStackKind::*;
        match kind {
            GlobalConstant(obj, ..) => obj.clone(),
            Constant(obj, ..) => obj.clone(),
            Var(obj, ..) => obj.clone(),
            CellVar(obj, ..) => obj.clone(),
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
                CellVar(..) => "CV",
                Temp(..) => "T",
                ReturnVal(..) => "R",
            };
            let obj = self.get_obj(kind);
            let obj = &*obj.read().unwrap();
            let top_marker = if i == 0 { "TOS" } else { "     " };
            let frame_marker =
                if i == self.call_frame_pointer() { "<----" } else { "" };
            let string =
                format!("{top_marker: <8}{kind_marker: <4}{obj:?}{frame_marker: >12}");
            items.push(string)
        }
        items.join("\n")
    }
}
