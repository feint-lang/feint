//! AST visitor for compiler.
use std::collections::HashSet;
use std::fmt;
use std::fmt::Formatter;

use crate::ast;
use crate::modules::std::STD;
use crate::op::{
    BinaryOperator, CompareOperator, InplaceOperator, ShortCircuitCompareOperator,
    UnaryOperator,
};
use crate::source::Location;
use crate::types::{new, ObjectRef};
use crate::vm::{globals, Code, Inst, PrintFlags};

use super::result::{CompErr, VisitResult};
use super::scope::{Scope, ScopeKind, ScopeTree};

pub struct Visitor {
    initial_scope_kind: ScopeKind,
    name: String,
    pub(crate) code: Code,
    pub(crate) scope_tree: ScopeTree,
    pub(crate) scope_depth: usize,
    pub(crate) func_nodes: Vec<(
        String, // name
        usize,  // address
        usize,  // scope tree pointer
        ast::Func,
    )>,
    global_names: HashSet<String>,
}

impl Visitor {
    fn new(
        initial_scope_kind: ScopeKind,
        name: &str,
        global_names: HashSet<String>,
    ) -> Self {
        assert!(matches!(initial_scope_kind, ScopeKind::Module | ScopeKind::Func));
        Self {
            initial_scope_kind,
            name: name.to_owned(),
            code: Code::default(),
            scope_tree: ScopeTree::new(initial_scope_kind),
            scope_depth: 0,
            func_nodes: vec![],
            global_names,
        }
    }

    pub(crate) fn for_module(name: &str, global_names: HashSet<String>) -> Self {
        Self::new(ScopeKind::Module, name, global_names)
    }

    pub(crate) fn for_func(name: &str, global_names: HashSet<String>) -> Self {
        Self::new(ScopeKind::Func, name, global_names)
    }

    // Entry Point Visitors --------------------------------------------

    pub(crate) fn visit_module(&mut self, node: ast::Module) -> VisitResult {
        if node.statements.is_empty() {
            self.push_nil();
            return Ok(());
        }
        self.visit_statements(node.statements)?;
        assert_eq!(self.scope_tree.pointer(), 0);
        self.fix_jumps()?;
        Ok(())
    }

    pub(crate) fn visit_func(&mut self, node: ast::Func) -> VisitResult {
        let params = node.params;

        // Return nil when the last statement is NOT an expression.
        let last_statement = node
            .block
            .statements
            .last()
            .expect("Block for function contains no statements");

        let return_nil = !matches!(last_statement.kind, ast::StatementKind::Expr(_));

        // Add var for this.
        self.scope_tree.add_var(0, "this", true);

        // Add vars for function parameters.
        let param_count = params.len();
        if param_count > 0 {
            let last = param_count - 1;
            for (i, name) in params.iter().enumerate() {
                if name.is_empty() {
                    if i == last {
                        self.scope_tree.add_var(0, "$args", true);
                    } else {
                        return Err(CompErr::var_args_must_be_last(
                            node.block.start,
                            node.block.end,
                        ));
                    }
                } else {
                    self.scope_tree.add_var(0, name, true);
                }
            }
        }

        self.visit_statements(node.block.statements)?;
        assert_eq!(self.scope_tree.pointer(), 0);
        assert!(self.scope_tree.in_func_scope());

        if return_nil {
            self.push_nil();
        }

        let return_addr = self.push(Inst::Return);

        // Update jump targets for labels.
        self.fix_jumps()?;

        // Update jump targets for explicit return statements.
        for addr in 0..return_addr {
            let inst = &self.code[addr];
            if let Inst::ReturnPlaceholder(inst_addr, depth) = inst {
                let rel_addr = return_addr - inst_addr;
                self.replace(*inst_addr, Inst::Jump(rel_addr, true, *depth));
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
            Kind::Break(expr) => self.visit_break(expr)?,
            Kind::Continue => self.visit_continue()?,
            Kind::Import(path) => self.visit_import(path)?,
            Kind::Jump(name) => {
                let jump_addr = self.push_placeholder(
                    Inst::Jump(0, true, 0),
                    "Jump address not set to label address",
                );
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
            Kind::Return(expr) => self.visit_return(expr)?,
            Kind::Halt(expr) => self.visit_halt(expr)?,
            Kind::Print(expr) => self.visit_print(expr)?,
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

    fn visit_import(&mut self, path: String) -> VisitResult {
        let var_name = path
            .split('.')
            .last()
            .expect("Import path should have at least one segment");
        self.scope_tree.add_var(self.len(), var_name, true);
        self.push(Inst::DeclareVar(var_name.to_owned()));
        self.push(Inst::LoadModule(path.to_owned()));
        self.push(Inst::AssignVar(var_name.to_owned()));
        Ok(())
    }

    fn visit_halt(&mut self, expr: ast::Expr) -> VisitResult {
        self.visit_expr(expr, None)?;
        self.push(Inst::HaltTop);
        Ok(())
    }

    fn visit_print(&mut self, expr: ast::Expr) -> VisitResult {
        if let Some(args) = expr.tuple_items() {
            let mut flags = PrintFlags::default();
            let n_args = args.len();
            if n_args > 0 && n_args <= 5 {
                let obj = args[0].clone();
                // XXX: This is kinda gnarly.
                if n_args > 1 && args[1].is_true() {
                    flags.insert(PrintFlags::ERR);
                }
                if n_args > 2 && args[2].is_true() {
                    flags.insert(PrintFlags::NL);
                }
                if n_args > 3 && args[3].is_true() {
                    flags.insert(PrintFlags::REPR);
                }
                if n_args > 4 && args[4].is_true() {
                    flags.insert(PrintFlags::NO_NIL);
                }
                self.visit_expr(obj, None)?;
                self.push(Inst::Print(flags));
                self.push_nil();
                return Ok(());
            }
        }
        let msg = concat!(
            "$print expects to receive at least 1 arg and no more ",
            "than 5 args: (object, ...flags)"
        );
        Err(CompErr::print(msg, expr.start, expr.end))
    }

    fn visit_return(&mut self, expr: ast::Expr) -> VisitResult {
        self.visit_expr(expr, None)?;
        self.push(Inst::ReturnPlaceholder(self.len(), self.scope_depth));
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
                let addr = self.push_placeholder(
                    Inst::LoadConst(0),
                    "Function constant index not updated",
                );
                let pointer = self.scope_tree.pointer();
                self.func_nodes.push((name, addr, pointer, func));
                self.push(Inst::CaptureSet(vec![]));
                self.push(Inst::MakeFunc);
            }
            Kind::Call(call) => self.visit_call(call)?,
            Kind::UnaryOp(op, b) => self.visit_unary_op(op, *b)?,
            Kind::BinaryOp(a, op, b) => self.visit_binary_op(*a, op, *b)?,
            Kind::CompareOp(a, op, b) => self.visit_compare_op(*a, op, *b)?,
            Kind::ShortCircuitCompareOp(a, op, b) => {
                self.visit_short_circuit_compare_op(*a, op, *b)?
            }
            Kind::InplaceOp(a, op, b) => self.visit_inplace_op(*a, op, *b)?,
        }
        Ok(())
    }

    fn visit_tuple(&mut self, items: Vec<ast::Expr>) -> VisitResult {
        if items.is_empty() {
            self.push_empty_tuple();
        } else {
            let num_items = items.len();
            self.visit_exprs(items)?;
            self.push(Inst::MakeTuple(num_items));
        }
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
            Kind::Always => self.push_always(),
            Kind::Ellipsis => self.push_nil(),
            Kind::Int(value) => {
                if let Some(index) = globals::shared_int_index(&value) {
                    self.push_global_const(index)
                } else {
                    self.add_const(new::int(value));
                }
            }
            Kind::Float(value) => {
                self.add_const(new::float(value));
            }
            Kind::String(value) => {
                if value.is_empty() {
                    self.push_empty_str();
                } else if value == "\n" {
                    self.push_newline();
                } else {
                    self.add_const(new::str(value));
                }
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
        if let Some(var) = self.scope_tree.find_var(&name, None) {
            if var.assigned {
                self.push(Inst::LoadVar(name, self.scope_depth - var.depth));
            } else {
                // This happens whenever an identifier appears on both
                // sides of an assignment, e.g. `x = x`, where the LHS
                // `x` has been declared by the assignment statement but
                // hasn't yet been assigned (there's a kind of dead zone
                // between declaration and assignment).
                //
                // Initially, the lookup of `x` on the RHS will find LHS
                // `x`, but RHS `x` actually refers to `x` in an outer
                // scope relative to the LHS `x`.
                //
                // In the simple `x = x` case, we can just search in the
                // parent of the current scope, but in more complex
                // cases, we have to ensure we search in the parent
                // scope of LHS `x`, which could be multiple levels up
                // from RHS `x`.
                if let Some(outer_var) = self.scope_tree.find_var_in_parent(&var) {
                    self.push(Inst::LoadVar(name, self.scope_depth - outer_var.depth));
                } else if self.is_module() {
                    if self.global_names.contains(&name) {
                        self.push(Inst::LoadGlobal(name));
                    } else if self.has_builtin(&name) {
                        self.push(Inst::LoadBuiltin(name));
                    } else {
                        return Err(CompErr::name_not_found(name, start, end));
                    }
                } else if self.is_func() {
                    self.code.add_free_var(name.as_str(), start, end);
                } else {
                    panic!("Unexpected scope type: {:?}", self.initial_scope_kind);
                }
            }
        } else if self.is_module() {
            // When compiling a module, all vars should resolve at this
            // point, so if the name doesn't resolve to a builtin,
            // that's an error.
            if self.global_names.contains(&name) {
                self.push(Inst::LoadGlobal(name));
            } else if self.has_builtin(&name) {
                self.push(Inst::LoadBuiltin(name));
            } else {
                return Err(CompErr::name_not_found(name, start, end));
            }
        } else if self.is_func() {
            // When compiling a function, vars may be defined in an
            // enclosing scope. These free vars will be resolved later.
            self.code.add_free_var(name.as_str(), start, end);
        } else {
            panic!("Unexpected scope type: {:?}", self.initial_scope_kind);
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
            self.visit_literal(ast::Literal::new_string(name.as_str()))?;
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
        assert!(
            !branches.is_empty() || default.is_some(),
            "At least one branch required for conditional"
        );

        // Addresses of branch jump-out instructions (added after each
        // branch's block). The target address for these isn't known
        // until the whole conditional suite is compiled.
        let mut jump_out_addrs: Vec<usize> = vec![];

        for (expr, block) in branches {
            self.enter_scope(ScopeKind::Block);

            // Evaluate branch expression.
            self.visit_expr(expr, None)?;

            // Placeholder for jump if branch condition is false.
            let jump_index = self.push_placeholder(
                Inst::JumpIfNot(0, true, 0),
                "Branch condition jump not set",
            );

            // Pop result of branch condition evaluation.
            self.push(Inst::Pop);

            // Branch selected. Execute body.
            self.visit_statements(block.statements)?;
            self.push(Inst::ScopeEnd); // NOTE: ScopeStart is at top of for loop

            // Placeholder for jump out of conditional suite if this
            // branch is selected.
            let jump_out_addr = self
                .push_placeholder(Inst::Jump(0, true, 0), "Branch jump out not set");
            jump_out_addrs.push(jump_out_addr);

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
            self.push(Inst::LoadVar(name, 0))
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
        let jump_out_addr = self
            .push_placeholder(Inst::JumpIfNot(0, true, 0), "Jump-out for loop not set");

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
            if name == "$main" && self.in_global_scope() {
                if self.scope_tree.find_var("$main", Some(0)).is_some() {
                    return Err(CompErr::cannot_reassign_special_ident(
                        name,
                        ident_expr.start,
                        ident_expr.end,
                    ));
                }
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
        self.scope_tree.add_var(self.len(), name.as_str(), false);
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
            if name == "$main" && !value_expr.is_func() {
                return Err(CompErr::main_must_be_func(
                    value_expr.start,
                    value_expr.end,
                ));
            }
            self.visit_expr(value_expr, Some(name.clone()))?;
            self.scope_tree.mark_assigned(self.scope_tree.pointer(), name.as_str());
            self.push(Inst::AssignVar(name));
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

    fn visit_short_circuit_compare_op(
        &mut self,
        expr_a: ast::Expr,
        op: ShortCircuitCompareOperator,
        expr_b: ast::Expr,
    ) -> VisitResult {
        use ShortCircuitCompareOperator::*;

        // LHS is always evaluated
        let lhs_is_bool = expr_a.is_bool();
        self.visit_expr(expr_a, None)?;

        match op {
            And => {
                // Force LHS to bool
                if !lhs_is_bool {
                    self.push(Inst::UnaryOp(UnaryOperator::AsBool));
                }

                // Skip RHS evaluation if LHS is false
                let jump_addr = self.push_placeholder(
                    Inst::JumpIfNot(0, true, 0),
                    "Jump target for && not updated",
                );

                // RHS evaluation
                self.visit_expr(expr_b, None)?;
                self.push(Inst::UnaryOp(UnaryOperator::AsBool));

                let jump_target = self.push(Inst::NoOp) - jump_addr;
                self.replace(jump_addr, Inst::JumpIfNot(jump_target, true, 0));

                self.push(Inst::NoOp);
            }
            Or => {
                // Force LHS to bool
                if !lhs_is_bool {
                    self.push(Inst::UnaryOp(UnaryOperator::AsBool));
                }

                // Skip RHS evaluation if LHS is true
                let jump_addr = self.push_placeholder(
                    Inst::JumpIf(0, true, 0),
                    "Jump target for || not updated",
                );

                // RHS evaluation
                self.visit_expr(expr_b, None)?;
                self.push(Inst::UnaryOp(UnaryOperator::AsBool));

                let jump_target = self.push(Inst::NoOp) - jump_addr;
                self.replace(jump_addr, Inst::JumpIf(jump_target, true, 0));
            }
            NilOr => {
                // Skip RHS evaluation if LHS is not nil
                let jump_addr = self.push_placeholder(
                    Inst::JumpIfNotNil(0, true, 0),
                    "Jump target for ?? not updated",
                );

                // RHS evaluation
                self.visit_expr(expr_b, None)?;

                let jump_target = self.push(Inst::NoOp) - jump_addr;
                self.replace(jump_addr, Inst::JumpIfNotNil(jump_target, true, 0));
            }
        }
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

    fn is_module(&self) -> bool {
        self.initial_scope_kind == ScopeKind::Module
    }

    fn is_func(&self) -> bool {
        self.initial_scope_kind == ScopeKind::Func
    }

    pub(crate) fn in_global_scope(&self) -> bool {
        self.scope_tree.in_global_scope()
    }

    fn has_builtin(&self, name: &str) -> bool {
        STD.read().unwrap().has_global(name)
    }

    fn len(&self) -> usize {
        self.code.len_chunk()
    }

    /// Push instruction and return its address.
    pub(crate) fn push(&mut self, inst: Inst) -> usize {
        let addr = self.len();
        self.code.push_inst(inst);
        addr
    }

    pub(crate) fn replace(&mut self, addr: usize, inst: Inst) {
        self.code.replace_inst(addr, inst);
    }

    fn push_placeholder(&mut self, inst: Inst, msg: &str) -> usize {
        self.push(Inst::Placeholder(self.len(), Box::new(inst), msg.to_owned()))
    }

    // Global constants ------------------------------------------------

    fn push_nil(&mut self) {
        self.push(Inst::LoadNil);
    }

    fn push_true(&mut self) {
        self.push(Inst::LoadTrue);
    }

    fn push_false(&mut self) {
        self.push(Inst::LoadFalse);
    }

    fn push_always(&mut self) {
        self.push(Inst::LoadAlways);
    }

    fn push_empty_str(&mut self) {
        self.push(Inst::LoadEmptyStr);
    }

    fn push_newline(&mut self) {
        self.push(Inst::LoadNewline);
    }

    fn push_empty_tuple(&mut self) {
        self.push(Inst::LoadEmptyTuple);
    }

    fn push_global_const(&mut self, index: usize) {
        self.push(Inst::LoadGlobalConst(index));
    }

    // Code unit constants ---------------------------------------------

    fn add_const(&mut self, obj: ObjectRef) -> usize {
        let index = self.code.add_const(obj);
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
        let suffix = if self.is_module() { "" } else { "()" };
        write!(f, "{}{suffix} with {} instructions", self.name, self.code.len_chunk())
    }
}

impl fmt::Debug for Visitor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self}: {:?}", self.code)
    }
}
