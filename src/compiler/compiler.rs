use std::fmt;
use std::fmt::Formatter;

use num_traits::ToPrimitive;

use crate::ast;
use crate::modules::BUILTINS;
use crate::types::{create, ObjectRef};
use crate::util::{
    BinaryOperator, CompareOperator, InplaceOperator, Location, Stack,
    UnaryCompareOperator, UnaryOperator,
};
use crate::vm::{Code, Inst};

use super::result::{CompErr, CompResult};
use super::scope::{Scope, ScopeKind, ScopeTree};

// Compiler ------------------------------------------------------------

pub struct Compiler {
    // The visitor stack is analogous to the VM call stack.
    visitor_stack: Stack<(Visitor, usize)>, // visitor, scope tree pointer
}

impl Compiler {
    pub fn new() -> Self {
        Self { visitor_stack: Stack::new() }
    }

    pub fn compile_script(
        &mut self,
        program: ast::Module,
        argv: Vec<&str>,
        keep_top_on_halt: bool,
    ) -> CompResult {
        let mut visitor = Visitor::for_module("$main");

        if program.statements.is_empty() {
            visitor.push(Inst::Halt(0));
            return Ok(visitor.code);
        }

        visitor.visit_module(program)?;

        assert!(
            visitor.scope_tree.in_global_scope(),
            "Expected to be in global scope after compiling script"
        );

        // Global Functions --------------------------------------------

        let mut has_main = false;
        let func_nodes = visitor.func_nodes.to_vec();
        self.visitor_stack.push((visitor, 0));
        for (addr, scope_tree_pointer, name, node) in func_nodes {
            if name == "$main" {
                has_main = true;
            }
            self.compile_func(addr, scope_tree_pointer, name, node)?;
        }
        let mut visitor = self.visitor_stack.pop().unwrap().0;

        // END Global Functions ----------------------------------------

        if has_main {
            let argc = argv.len();
            for arg in argv {
                visitor.add_const(create::new_str(arg));
            }
            visitor.push(Inst::LoadVar("$main".to_string()));
            visitor.push(Inst::Call(argc));
            visitor.push(Inst::HaltTop);
        } else {
            if !keep_top_on_halt {
                visitor.push(Inst::Pop);
            }
            visitor.push(Inst::Halt(0));
        }

        Ok(visitor.code)
    }

    /// Compile a function and inject it into the *parent* visitor at
    /// the specified address.
    fn compile_func(
        &mut self,
        // Address in parent visitor's code where function was defined.
        func_addr: usize,
        // Pointer to scope in parent where function was defined. This
        // is needed so that we can start the search for cell vars in
        // the correct scope in the parent visitor.
        parent_scope_pointer: usize,
        name: String,
        node: ast::Func,
    ) -> VisitResult {
        let stack = &mut self.visitor_stack;
        let params = node.params.clone();

        let mut visitor = Visitor::for_func(name.as_str());
        visitor.visit_func(node)?;

        // Unresolved names are assumed to be builtins.
        let builtins = BUILTINS.read().unwrap();
        let mut presumed_builtins = vec![];

        // Captured vars in current function. These are free vars in the
        // current function that are defined in an enclosing function or
        // module.
        //
        // Each entry contains the address of the free var in the
        // current function and the local var index in the enclosing
        // function.
        let mut captured = vec![];

        for (addr, name, start, end) in visitor.code.free_vars().iter() {
            let mut found = false;
            let mut current_scope_pointer = parent_scope_pointer;

            log::trace!("Searching for free var: {name}");

            for (up_visitor, up_scope_pointer) in stack.iter() {
                let result = up_visitor
                    .scope_tree
                    .find_var(name.as_str(), Some(current_scope_pointer));

                if let Some((_, name, _)) = result {
                    log::trace!("Found free var: {name}");
                    found = true;
                    captured.push((*addr, name));
                    break;
                }

                current_scope_pointer = *up_scope_pointer;
            }

            if !found {
                presumed_builtins.push((*addr, name.to_owned(), *start, *end));
            }
        }

        for (addr, name, start, end) in presumed_builtins.into_iter() {
            if builtins.has_name(name.as_str()) {
                visitor.replace(addr, Inst::LoadVar(name.to_owned()));
            } else {
                return Err(CompErr::name_not_found(name.to_owned(), start, end));
            }
        }

        for (addr, name) in captured.iter() {
            visitor.replace(*addr, Inst::LoadCaptured(name.to_string()));
        }

        // Inner Functions ---------------------------------------------

        let func_nodes = visitor.func_nodes.to_vec();
        self.visitor_stack.push((visitor, parent_scope_pointer));
        for (addr, scope_tree_pointer, name, node) in func_nodes {
            self.compile_func(addr, scope_tree_pointer, name, node)?;
        }
        let visitor = self.visitor_stack.pop().unwrap().0;

        // END Inner Functions -----------------------------------------

        let func = create::new_func(name, params, visitor.code);

        let parent_visitor = &mut self.visitor_stack.peek_mut().unwrap().0;
        let const_index = parent_visitor.code.add_const(func);

        if captured.is_empty() {
            parent_visitor.replace(func_addr, Inst::LoadConst(const_index));
        } else {
            let captured = captured.iter().map(|(_, name)| name.to_string()).collect();
            parent_visitor.replace(func_addr, Inst::MakeClosure(const_index, captured));
        }

        Ok(())
    }
}

// Visitor -------------------------------------------------------------

type VisitResult = Result<(), CompErr>;

pub struct Visitor {
    initial_scope_kind: ScopeKind,
    code: Code,
    scope_tree: ScopeTree,
    scope_depth: usize,
    func_nodes: Vec<(
        usize,  // address
        usize,  // scope tree pointer
        String, // name
        ast::Func,
    )>,
    name: String,
}

impl Visitor {
    fn new(initial_scope_kind: ScopeKind, name: &str) -> Self {
        assert!(matches!(initial_scope_kind, ScopeKind::Module | ScopeKind::Func));
        Self {
            initial_scope_kind,
            code: Code::new(),
            scope_tree: ScopeTree::new(initial_scope_kind),
            scope_depth: 0,
            func_nodes: vec![],
            name: name.to_owned(),
        }
    }

    fn for_module(name: &str) -> Self {
        Self::new(ScopeKind::Module, name)
    }

    fn for_func(name: &str) -> Self {
        Self::new(ScopeKind::Func, name)
    }

    // Entry Point Visitors --------------------------------------------

    fn visit_module(&mut self, node: ast::Module) -> VisitResult {
        self.visit_statements(node.statements)?;
        assert_eq!(self.scope_tree.pointer(), 0);
        self.fix_jumps()?;
        Ok(())
    }

    fn visit_func(&mut self, node: ast::Func) -> VisitResult {
        let params = node.params;

        // Return nil when the last statement is NOT an expression.
        let last_statement = node
            .block
            .statements
            .last()
            .expect("Block for function contains no statements");

        let return_nil = !matches!(last_statement.kind, ast::StatementKind::Expr(_));

        // Add var for this.
        self.scope_tree.add_var("this", true);

        // Add vars for function parameters.
        let param_count = params.len();
        if param_count > 0 {
            let last = param_count - 1;
            for (i, name) in params.iter().enumerate() {
                if name.is_empty() {
                    if i == last {
                        self.scope_tree.add_var("$args", true);
                    } else {
                        return Err(CompErr::var_args_must_be_last(
                            node.block.start,
                            node.block.end,
                        ));
                    }
                } else {
                    self.scope_tree.add_var(name, true);
                }
            }
        }

        self.visit_statements(node.block.statements)?;
        assert_eq!(self.scope_tree.pointer(), 0);
        assert!(self.scope_tree.in_func_scope());

        if return_nil {
            self.push_nil();
        }

        let return_addr = self.len();
        self.push(Inst::Return);

        // Update jump targets for labels.
        self.fix_jumps()?;

        // Update jump targets for explicit return statements.
        for addr in 0..return_addr {
            let inst = &self.code[addr];
            if let Inst::ReturnPlaceholder(inst_addr, depth) = inst {
                let rel_addr = return_addr - inst_addr;
                self.replace(*inst_addr, Inst::Jump(rel_addr, true, depth - 1));
            }
        }

        Ok(())
    }

    /// This pushes the args onto the stack first and then the function.
    fn visit_call(&mut self, node: ast::Call) -> VisitResult {
        let callable = node.callable;
        let args = node.args;
        let num_args = args.len();
        self.visit_exprs(args)?;
        self.visit_expr(*callable, None)?;
        self.push(Inst::Call(num_args));
        Ok(())
    }

    // Visitors --------------------------------------------------------

    fn visit_statements(&mut self, statements: Vec<ast::Statement>) -> VisitResult {
        let num_statements = statements.len();
        if num_statements > 0 {
            let last = num_statements - 1;
            for (i, statement) in statements.into_iter().enumerate() {
                self.push(Inst::StatementStart(statement.start, statement.end));
                self.visit_statement(statement)?;
                if i != last {
                    self.push(Inst::Pop);
                }
            }
        }
        Ok(())
    }

    fn visit_statement(&mut self, node: ast::Statement) -> VisitResult {
        type Kind = ast::StatementKind;
        match node.kind {
            Kind::Jump(name) => {
                let jump_addr = self.len();
                self.push(Inst::Placeholder(
                    0,
                    Box::new(Inst::Jump(0, true, 0)),
                    "Jump address not set to label address".to_owned(),
                ));
                self.scope_tree.add_jump(name.as_str(), jump_addr);
            }
            Kind::Label(name, expr) => {
                let addr = self.len();
                self.visit_expr(expr, None)?;
                if self.scope_tree.add_label(name.as_str(), addr).is_some() {
                    return Err(CompErr::duplicate_label_in_scope(
                        name, node.start, node.end,
                    ));
                }
            }
            Kind::Break(expr) => self.visit_break(expr)?,
            Kind::Continue => self.visit_continue()?,
            Kind::Return(expr) => self.visit_return(expr)?,
            Kind::Import(name) => self.visit_import(name)?,
            Kind::Expr(expr) => self.visit_expr(expr, None)?,
        }
        Ok(())
    }

    fn visit_break(&mut self, expr: ast::Expr) -> VisitResult {
        self.visit_expr(expr, None)?;
        self.push(Inst::BreakPlaceholder(self.len(), self.scope_depth));
        Ok(())
    }

    fn visit_continue(&mut self) -> VisitResult {
        self.push(Inst::ContinuePlaceholder(self.len(), self.scope_depth));
        Ok(())
    }

    fn visit_return(&mut self, expr: ast::Expr) -> VisitResult {
        self.visit_expr(expr, None)?;
        self.push(Inst::ReturnPlaceholder(self.len(), self.scope_depth));
        Ok(())
    }

    fn visit_import(&mut self, name: String) -> VisitResult {
        self.scope_tree.add_var(name.as_str(), true);
        self.push(Inst::LoadModule(name));
        Ok(())
    }

    fn visit_exprs(&mut self, exprs: Vec<ast::Expr>) -> VisitResult {
        for expr in exprs {
            self.visit_expr(expr, None)?;
        }
        Ok(())
    }

    /// Visit an expression. The `name` argument is currently only
    /// used to assign names to functions.
    fn visit_expr(&mut self, node: ast::Expr, name: Option<String>) -> VisitResult {
        type Kind = ast::ExprKind;
        match node.kind {
            Kind::Tuple(items) => self.visit_tuple(items)?,
            Kind::List(items) => self.visit_list(items)?,
            Kind::Map(entries) => self.visit_map(entries)?,
            Kind::Literal(literal) => self.visit_literal(literal)?,
            Kind::FormatString(items) => self.visit_format_string(items)?,
            Kind::Ident(ident) => self.visit_ident(ident, node.start, node.end)?,
            Kind::DeclarationAndAssignment(lhs_expr, value_expr) => {
                self.visit_declaration(*lhs_expr.clone())?;
                self.visit_assignment(*lhs_expr, *value_expr)?
            }
            Kind::Assignment(lhs_expr, value_expr) => {
                self.visit_assignment(*lhs_expr, *value_expr)?
            }
            Kind::Block(block) => self.visit_block(block)?,
            Kind::Conditional(branches, default) => {
                self.visit_conditional(branches, default)?
            }
            Kind::Loop(expr, block) => self.visit_loop(*expr, block)?,
            Kind::Func(func) => {
                let name = name.map_or_else(|| "<anonymous>".to_owned(), |name| name);
                let addr = self.len();
                let pointer = self.scope_tree.pointer();
                self.func_nodes.push((addr, pointer, name, func));
                self.push(Inst::Placeholder(
                    addr,
                    Box::new(Inst::LoadConst(0)),
                    "Function placeholder not updated".to_owned(),
                ));
            }
            Kind::Call(call) => self.visit_call(call)?,
            Kind::UnaryOp(op, b) => self.visit_unary_op(op, *b)?,
            Kind::UnaryCompareOp(op, b) => self.visit_unary_compare_op(op, *b)?,
            Kind::BinaryOp(a, op, b) => self.visit_binary_op(*a, op, *b)?,
            Kind::CompareOp(a, op, b) => self.visit_compare_op(*a, op, *b)?,
            Kind::InplaceOp(a, op, b) => self.visit_inplace_op(*a, op, *b)?,
        }
        Ok(())
    }

    fn visit_tuple(&mut self, items: Vec<ast::Expr>) -> VisitResult {
        let num_items = items.len();
        self.visit_exprs(items)?;
        self.push(Inst::MakeTuple(num_items));
        Ok(())
    }

    fn visit_list(&mut self, items: Vec<ast::Expr>) -> VisitResult {
        let num_items = items.len();
        self.visit_exprs(items)?;
        self.push(Inst::MakeList(num_items));
        Ok(())
    }

    fn visit_map(&mut self, entries: Vec<(ast::Expr, ast::Expr)>) -> VisitResult {
        let num_items = entries.len();
        for (name, val) in entries {
            self.visit_expr(name, None)?;
            self.visit_expr(val, None)?;
        }
        self.push(Inst::MakeMap(num_items * 2));
        Ok(())
    }

    fn visit_literal(&mut self, node: ast::Literal) -> VisitResult {
        type Kind = ast::LiteralKind;
        match node.kind {
            Kind::Nil => self.push_nil(),
            Kind::Bool(true) => self.push_true(),
            Kind::Bool(false) => self.push_false(),
            Kind::Ellipsis => self.push_nil(),
            Kind::Int(value) => {
                if create::in_shared_int_range(&value) {
                    let index = value.to_usize().unwrap() + 3;
                    self.push(Inst::LoadGlobalConst(index))
                } else {
                    self.add_const(create::new_int(value));
                }
            }
            Kind::Float(value) => {
                self.add_const(create::new_float(value));
            }
            Kind::String(value) => {
                self.add_const(create::new_str(value));
            }
        }
        Ok(())
    }

    fn visit_format_string(&mut self, items: Vec<ast::Expr>) -> VisitResult {
        let num_items = items.len();
        self.visit_exprs(items)?;
        self.push(Inst::MakeString(num_items));
        Ok(())
    }

    /// Visit identifier (AKA name) as expression (i.e., not as part of
    /// an assignment).
    fn visit_ident(
        &mut self,
        node: ast::Ident,
        start: Location,
        end: Location,
    ) -> VisitResult {
        let name = node.name();
        // NOTE: When a function is being compiled, find_var will
        //       traverse up as far as the top level scope of the
        //       function. It will NOT proceed up into a function's
        //       enclosing scope, whether that's an outer function or
        //       a module.
        match self.scope_tree.find_var(name.as_str(), None) {
            Some((.., true)) => {
                self.push(Inst::LoadVar(name));
            }
            // The var exists but has not yet been assigned--where the
            // RHS side name shadows a name from an outer scope. E.g.:
            //
            //     x = "global"
            //     block -> x = x
            //     f = () -> x = x
            // defined in an enclosing scope.
            Some((.., false)) => {
                if self.initial_scope_kind == ScopeKind::Module {
                    // When compiling a module, the RHS must be defined
                    // in an outer scope. If it isn't, that's an error.
                    if self.scope_tree.find_var_in_parent(name.as_str()).is_some() {
                        self.push(Inst::LoadOuterVar(name));
                    } else {
                        if BUILTINS.read().unwrap().has_name(name.as_str()) {
                            self.push(Inst::LoadOuterVar(name));
                        } else {
                            return Err(CompErr::name_not_found(name, start, end));
                        }
                    }
                } else if self.initial_scope_kind == ScopeKind::Func {
                    // When compiling a function, the RHS is considered
                    // a free var and will be resolved later.
                    self.code.add_free_var(name.as_str(), start, end);
                }
            }
            // Var wasn't found in current scope or its ancestors.
            None => {
                if self.initial_scope_kind == ScopeKind::Module {
                    // When compiling a module, all vars should resolve
                    // at this stage, so this is an error.
                    if BUILTINS.read().unwrap().has_name(name.as_str()) {
                        self.push(Inst::LoadVar(name));
                    } else {
                        return Err(CompErr::name_not_found(name, start, end));
                    }
                } else if self.initial_scope_kind == ScopeKind::Func {
                    // When compiling a function, vars may be defined in
                    // an enclosing scope. These free vars will be
                    // resolved later.
                    self.code.add_free_var(name.as_str(), start, end);
                } else {
                    panic!("Unexpected scope type: {:?}", self.initial_scope_kind);
                }
            }
        }
        Ok(())
    }

    fn visit_get_attr(
        &mut self,
        obj_expr: ast::Expr,
        name_expr: ast::Expr,
    ) -> VisitResult {
        self.visit_expr(obj_expr, None)?;
        if let Some(name) = name_expr.ident_name() {
            self.visit_literal(ast::Literal::new_string(name))?;
        } else {
            self.visit_expr(name_expr, None)?;
        }
        self.push(Inst::BinaryOp(BinaryOperator::Dot));
        Ok(())
    }

    fn visit_block(&mut self, node: ast::StatementBlock) -> VisitResult {
        self.enter_scope(ScopeKind::Block);
        self.visit_statements(node.statements)?;
        self.exit_scope();
        Ok(())
    }

    fn visit_conditional(
        &mut self,
        branches: Vec<(ast::Expr, ast::StatementBlock)>,
        default: Option<ast::StatementBlock>,
    ) -> VisitResult {
        assert!(!branches.is_empty(), "At least one branch required for conditional");

        // Addresses of branch jump-out instructions (added after each
        // branch's block). The target address for these isn't known
        // until the whole conditional suite is compiled.
        let mut jump_out_addrs: Vec<usize> = vec![];

        for (expr, block) in branches {
            self.enter_scope(ScopeKind::Block);

            // Evaluate branch expression.
            self.visit_expr(expr, None)?;

            // Placeholder for jump if branch condition is false.
            let jump_index = self.len();
            self.push(Inst::Placeholder(
                jump_index,
                Box::new(Inst::JumpIfNot(0, true, 0)),
                "Branch condition jump not set".to_owned(),
            ));

            // Pop result of branch condition evaluation.
            self.push(Inst::Pop);

            // Branch selected. Execute body.
            self.visit_statements(block.statements)?;
            self.push(Inst::ScopeEnd); // NOTE: ScopeStart is at top of for loop

            // Placeholder for jump out of conditional suite if this
            // branch is selected.
            let jump_out_addr = self.len();
            jump_out_addrs.push(jump_out_addr);
            self.push(Inst::Placeholder(
                jump_out_addr,
                Box::new(Inst::Jump(0, true, 0)),
                "Branch jump out not set".to_owned(),
            ));

            // Set jump target for when branch condition is false.
            let rel_addr = self.len() - jump_index;
            self.replace(jump_index, Inst::JumpIfNot(rel_addr, true, 0));

            // If branch condition evaluated false, replace with nil.
            // The branch *has* to return something, even when the
            // branch condition is false.
            self.push(Inst::Pop);
            self.push_nil();

            self.exit_scope();
        }

        // Default block (if present).
        if let Some(default_block) = default {
            self.visit_block(default_block)?;
        } else {
            self.push_nil();
        }

        // Address of instruction after conditional suite.
        let after_addr = self.len();

        // Replace jump-out placeholders with actual jumps.
        for addr in jump_out_addrs {
            let rel_addr = after_addr - addr;
            self.replace(addr, Inst::Jump(rel_addr, true, 0));
        }

        Ok(())
    }

    fn visit_loop(
        &mut self,
        expr: ast::Expr,
        block: ast::StatementBlock,
    ) -> VisitResult {
        use ast::ExprKind::DeclarationAndAssignment;

        // Enter scope *before* loop condition.
        self.enter_scope(ScopeKind::Block);

        let loop_scope_depth = self.scope_depth;

        // Evaluate loop expression. If the expression is an assignment,
        // evaluate the value of the var instead.
        let loop_addr = if let DeclarationAndAssignment(lhs, val) = expr.kind {
            let name = if let Some(name) = lhs.ident_name() {
                name
            } else {
                return Err(CompErr::expected_ident(lhs.start, lhs.end));
            };
            self.visit_declaration(*lhs.clone())?;
            self.visit_assignment(*lhs, *val)?;
            let loop_addr = self.len();
            self.push(Inst::LoadVar(name));
            loop_addr
        } else {
            let loop_addr = self.len();
            if expr.is_false() {
                self.push_nil();
            } else {
                self.visit_expr(expr, None)?;
            }
            loop_addr
        };

        // Placeholder for jump-out if loop expression evaluates false.
        let jump_out_addr = self.len();
        self.push(Inst::Placeholder(
            jump_out_addr,
            Box::new(Inst::JumpIfNot(0, true, 0)),
            "Jump-out for loop not set".to_owned(),
        ));

        // Pop result of loop condition evaluation.
        self.push(Inst::Pop);

        // Run the loop body.
        let block_start_addr = self.len();
        self.visit_statements(block.statements)?;
        let block_end_addr = self.len();

        // Jump to top of loop.
        let rel_addr = self.len() - loop_addr;
        self.push(Inst::Jump(rel_addr, false, 0));

        // Jump-out target address.
        let jump_out_target = self.len();

        // NOTE: Exit scope *after* jumping out.
        self.exit_scope();

        // Set target of jump-out placeholder.
        let rel_addr = jump_out_target - jump_out_addr;
        self.replace(jump_out_addr, Inst::JumpIfNot(rel_addr, true, 0));

        // Set address of breaks and continues.
        for addr in block_start_addr..=block_end_addr {
            let inst = &self.code[addr];
            if let Inst::BreakPlaceholder(inst_addr, depth) = inst {
                let rel_addr = jump_out_target - addr;
                let scope_exit_count = depth - loop_scope_depth;
                let inst = Inst::Jump(rel_addr, true, scope_exit_count);
                self.replace(*inst_addr, inst);
            } else if let Inst::ContinuePlaceholder(inst_addr, depth) = inst {
                let rel_addr = addr - loop_addr;
                let scope_exit_count = depth - loop_scope_depth;
                let inst = Inst::JumpPushNil(rel_addr, false, scope_exit_count);
                self.replace(*inst_addr, inst);
            }
        }

        Ok(())
    }

    fn visit_unary_op(&mut self, op: UnaryOperator, expr: ast::Expr) -> VisitResult {
        self.visit_expr(expr, None)?;
        self.push(Inst::UnaryOp(op));
        Ok(())
    }

    fn visit_unary_compare_op(
        &mut self,
        op: UnaryCompareOperator,
        expr: ast::Expr,
    ) -> VisitResult {
        self.visit_expr(expr, None)?;
        self.push(Inst::UnaryCompareOp(op));
        Ok(())
    }

    fn visit_binary_op(
        &mut self,
        expr_a: ast::Expr,
        op: BinaryOperator,
        expr_b: ast::Expr,
    ) -> VisitResult {
        use BinaryOperator::*;
        match op {
            Dot => self.visit_get_attr(expr_a, expr_b),
            _ => {
                self.visit_expr(expr_a, None)?;
                self.visit_expr(expr_b, None)?;
                self.push(Inst::BinaryOp(op));
                Ok(())
            }
        }
    }

    fn visit_declaration(&mut self, ident_expr: ast::Expr) -> VisitResult {
        let name = if let Some(name) = ident_expr.is_ident() {
            if name == "this" {
                return Err(CompErr::cannot_assign_special_ident(
                    name,
                    ident_expr.start,
                    ident_expr.end,
                ));
            }
            name
        } else if let Some(name) = ident_expr.is_special_ident() {
            if name == "$main" && self.scope_tree.in_global_scope() {
                name
            } else {
                return Err(CompErr::cannot_assign_special_ident(
                    name,
                    ident_expr.start,
                    ident_expr.end,
                ));
            }
        } else if let Some(_name) = ident_expr.is_type_ident() {
            todo!("Implement custom types")
        } else {
            return Err(CompErr::expected_ident(ident_expr.start, ident_expr.end));
        };
        self.scope_tree.add_var(name.as_str(), false);
        self.push(Inst::DeclareVar(name));
        Ok(())
    }

    fn visit_assignment(
        &mut self,
        lhs_expr: ast::Expr,
        value_expr: ast::Expr,
    ) -> VisitResult {
        // TODO: Allow assignment to attributes
        if let Some(name) = lhs_expr.ident_name() {
            self.visit_expr(value_expr, Some(name.clone()))?;
            match self.scope_tree.find_var(name.as_str(), None) {
                Some((pointer, n, _)) => {
                    self.scope_tree.mark_assigned(pointer, n.as_str());
                    self.push(Inst::AssignVar(name))
                }
                None => self.push(Inst::AssignVar(name)),
            }
            Ok(())
        } else {
            Err(CompErr::expected_ident(lhs_expr.start, lhs_expr.end))
        }
    }

    fn visit_compare_op(
        &mut self,
        expr_a: ast::Expr,
        op: CompareOperator,
        expr_b: ast::Expr,
    ) -> VisitResult {
        self.visit_expr(expr_a, None)?;
        self.visit_expr(expr_b, None)?;
        self.push(Inst::CompareOp(op));
        Ok(())
    }

    fn visit_inplace_op(
        &mut self,
        expr_a: ast::Expr,
        op: InplaceOperator,
        expr_b: ast::Expr,
    ) -> VisitResult {
        // TODO: Allow in place attribute updates
        if expr_a.is_ident().is_none() {
            return Err(CompErr::expected_ident(expr_a.start, expr_a.end));
        }
        self.visit_expr(expr_a, None)?;
        self.visit_expr(expr_b, None)?;
        self.push(Inst::InplaceOp(op));
        Ok(())
    }

    // Utilities -------------------------------------------------------

    fn len(&self) -> usize {
        self.code.len_chunk()
    }

    fn push(&mut self, inst: Inst) {
        self.code.push_inst(inst);
    }

    fn replace(&mut self, addr: usize, inst: Inst) {
        self.code.replace_inst(addr, inst);
    }

    // Global constants ------------------------------------------------

    fn push_nil(&mut self) {
        self.push(Inst::LoadNil)
    }

    fn push_true(&mut self) {
        self.push(Inst::LoadTrue)
    }

    fn push_false(&mut self) {
        self.push(Inst::LoadFalse)
    }

    // Code unit constants ---------------------------------------------

    fn add_const(&mut self, val: ObjectRef) -> usize {
        let index = self.code.add_const(val);
        self.push(Inst::LoadConst(index));
        index
    }

    // Scopes ----------------------------------------------------------

    /// Add nested scope to current scope then make the new scope the
    /// current scope.
    fn enter_scope(&mut self, kind: ScopeKind) {
        self.push(Inst::ScopeStart);
        self.scope_tree.add(kind);
        self.scope_depth += 1;
    }

    /// Move up to the parent scope of the current scope.
    fn exit_scope(&mut self) {
        self.push(Inst::ScopeEnd);
        self.scope_tree.move_up();
        self.scope_depth -= 1;
    }

    /// Update jump instructions with their target label addresses.
    fn fix_jumps(&mut self) -> VisitResult {
        let code = &mut self.code;
        let scope_tree = &self.scope_tree;
        let mut not_found: Option<String> = None;
        let mut jump_out_of_func: Option<String> = None;
        scope_tree.walk_up(&mut |scope: &Scope, jump_depth: usize| {
            for (name, jump_addr) in scope.jumps().iter() {
                let result = scope.find_label(scope_tree, name, None);
                if let Some((label_addr, label_depth)) = result {
                    let rel_addr = label_addr - jump_addr;
                    let depth = jump_depth - label_depth;
                    let new_inst = Inst::JumpPushNil(rel_addr, true, depth);
                    code.replace_inst(*jump_addr, new_inst);
                } else {
                    if scope.is_func() {
                        jump_out_of_func = Some(name.clone());
                    } else {
                        not_found = Some(name.clone());
                    }
                    return false;
                }
            }
            true
        });
        // TODO: Fix locations
        if let Some(name) = jump_out_of_func {
            return Err(CompErr::cannot_jump_out_of_func(
                name,
                Location::default(),
                Location::default(),
            ));
        } else if let Some(name) = not_found {
            return Err(CompErr::label_not_found_in_scope(
                name,
                Location::default(),
                Location::default(),
            ));
        }
        Ok(())
    }
}

impl fmt::Display for Visitor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let suffix =
            if self.initial_scope_kind == ScopeKind::Module { "" } else { "()" };
        write!(f, "{}{suffix} with {} instructions", self.name, self.code.len_chunk())
    }
}

impl fmt::Debug for Visitor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self}: {:?}", self.code)
    }
}
