use crate::ast;
use crate::types::ObjectRef;
use crate::util::{BinaryOperator, UnaryOperator};
use crate::vm::{Chunk, Inst, RuntimeContext, VM};

use super::result::{CompErr, CompResult};
use super::scope::{Scope, ScopeKind, ScopeTree};

// Compiler ------------------------------------------------------------

/// Compile AST to VM instructions.
pub fn compile(vm: &mut VM, program: ast::Program) -> CompResult {
    let mut visitor = Visitor::new(&mut vm.ctx);
    visitor.visit_program(program)?;
    Ok(visitor.chunk)
}

// Visitor -------------------------------------------------------------

type VisitResult = Result<(), CompErr>;

struct Visitor<'a> {
    ctx: &'a mut RuntimeContext,
    chunk: Chunk,
    scope_tree: ScopeTree,
    scope_depth: usize,
    has_main: bool,
}

impl<'a> Visitor<'a> {
    fn new(ctx: &'a mut RuntimeContext) -> Self {
        Self {
            ctx,
            chunk: Chunk::new(),
            scope_tree: ScopeTree::new(),
            scope_depth: 0,
            has_main: false,
        }
    }

    // Visitors --------------------------------------------------------

    fn visit_program(&mut self, node: ast::Program) -> VisitResult {
        for statement in node.statements {
            self.visit_statement(statement)?;
        }
        assert_eq!(self.scope_tree.pointer(), 0);
        self.fix_jumps()?;
        if self.has_main {
            self.push(Inst::LoadVar("$main".to_string()));
            self.push(Inst::Call(0));
            self.push(Inst::Return);
            self.push(Inst::HaltTop);
        } else {
            self.push(Inst::Halt(0));
        }
        Ok(())
    }

    fn visit_statement(&mut self, node: ast::Statement) -> VisitResult {
        type Kind = ast::StatementKind;
        match node.kind {
            Kind::Jump(name) => {
                let jump_addr = self.chunk.len();
                self.push(Inst::Placeholder(
                    0,
                    Box::new(Inst::Jump(0, 0)),
                    "Jump address not set to label address".to_owned(),
                ));
                self.scope_tree.add_jump(name.as_str(), jump_addr);
            }
            Kind::Label(name) => {
                self.push(Inst::NoOp);
                let addr = self.chunk.len() - 1;
                if self.scope_tree.add_label(name.as_str(), addr).is_some() {
                    return Err(CompErr::new_duplicate_label_in_scope(name));
                }
            }
            Kind::Continue => self.visit_continue()?,
            Kind::Expr(expr) => self.visit_expr(expr)?,
        }
        Ok(())
    }

    fn visit_expr(&mut self, node: ast::Expr) -> VisitResult {
        type Kind = ast::ExprKind;
        match node.kind {
            Kind::Tuple(items) => self.visit_tuple(items)?,
            Kind::Literal(literal) => self.visit_literal(literal)?,
            Kind::FormatString(items) => self.visit_format_string(items)?,
            Kind::Ident(ident) => self.visit_ident(ident)?,
            Kind::Block(block) => self.visit_block(block, ScopeKind::Block)?,
            Kind::Conditional(branches, default) => {
                self.visit_conditional(branches, default, ScopeKind::Block)?
            }
            Kind::Loop(expr, block) => {
                self.visit_loop(*expr, block, ScopeKind::Block)?
            }
            Kind::Break(expr) => self.visit_break(*expr)?,
            Kind::Func(func) => self.visit_func(func)?,
            Kind::Call(call) => self.visit_call(call)?,
            Kind::UnaryOp(op, b) => self.visit_unary_op(op, *b)?,
            Kind::BinaryOp(a, op, b) => self.visit_binary_op(*a, op, *b)?,
        }
        Ok(())
    }

    fn visit_tuple(&mut self, items: Vec<ast::Expr>) -> VisitResult {
        let num_items = items.len();
        for item in items {
            self.visit_expr(item)?;
        }
        self.push(Inst::MakeTuple(num_items));
        Ok(())
    }

    fn visit_literal(&mut self, node: ast::Literal) -> VisitResult {
        type Kind = ast::LiteralKind;
        match node.kind {
            Kind::Nil => self.push_const(0),
            Kind::Bool(true) => self.push_const(1),
            Kind::Bool(false) => self.push_const(2),
            Kind::Int(value) => self.add_const(self.ctx.builtins.new_int(value)),
            Kind::Float(value) => self.add_const(self.ctx.builtins.new_float(value)),
            Kind::String(value) => self.add_const(self.ctx.builtins.new_string(value)),
        }
        Ok(())
    }

    fn visit_format_string(&mut self, items: Vec<ast::Expr>) -> VisitResult {
        let num_items = items.len();
        for item in items {
            self.visit_expr(item)?;
        }
        self.push(Inst::MakeString(num_items));
        Ok(())
    }

    // Visit identifier as expression (i.e., not as part of an
    // assignment).
    fn visit_ident(&mut self, node: ast::Ident) -> VisitResult {
        type Kind = ast::IdentKind;
        match node.kind {
            Kind::Ident(name) => self.push(Inst::LoadVar(name)),
            Kind::TypeIdent(name) => self.push(Inst::LoadVar(name)),
        }
        Ok(())
    }

    fn visit_get_attr(
        &mut self,
        obj_expr: ast::Expr,
        name_expr: ast::Expr,
    ) -> VisitResult {
        use ast::ExprKind;
        use ast::{Ident, IdentKind};
        self.visit_expr(obj_expr)?;
        let kind = &name_expr.kind;
        if let ExprKind::Ident(Ident { kind: IdentKind::Ident(name) }) = kind {
            self.visit_literal(ast::Literal::new_string(name))?;
        } else {
            self.visit_expr(name_expr)?;
        }
        self.push(Inst::BinaryOp(BinaryOperator::Dot));
        Ok(())
    }

    fn visit_assignment(
        &mut self,
        name_expr: ast::Expr,
        value_expr: ast::Expr,
    ) -> VisitResult {
        use ast::ExprKind;
        use ast::{Ident, IdentKind};
        let kind = name_expr.kind;
        if let ExprKind::Ident(Ident { kind: IdentKind::Ident(name) }) = kind {
            self.push(Inst::DeclareVar(name.clone()));
            self.visit_expr(value_expr)?;
            self.push(Inst::AssignVar(name));
        } else {
            return Err(CompErr::new_expected_ident());
        }
        Ok(())
    }

    fn visit_block(&mut self, node: ast::Block, scope_kind: ScopeKind) -> VisitResult {
        self.push(Inst::ScopeStart);
        self.enter_scope(scope_kind);
        for statement in node.statements {
            self.visit_statement(statement)?;
        }
        self.push(Inst::ScopeEnd);
        self.exit_scope();
        Ok(())
    }

    fn visit_conditional(
        &mut self,
        branches: Vec<(ast::Expr, ast::Block)>,
        default: Option<ast::Block>,
        scope_kind: ScopeKind,
    ) -> VisitResult {
        assert!(branches.len() > 0, "At least one branch required for conditional");

        // Addresses of branch jump-out instructions (added after each
        // branch's block). The target address for these isn't known
        // until the whole conditional suite is compiled.
        let mut jump_out_addrs: Vec<usize> = vec![];

        for (expr, block) in branches {
            // Evaluate branch expression.
            self.visit_expr(expr)?;

            // Placeholder for jump depending on result of branch expr.
            let jump_index = self.chunk.len();
            self.push(Inst::Placeholder(
                jump_index,
                Box::new(Inst::JumpIfElse(0, 0, 0)),
                "Branch condition jump not set".to_owned(),
            ));

            // Start of branch block (jump target if branch condition is
            // true).
            let block_addr = jump_index + 1;
            self.visit_block(block, scope_kind)?;

            // Placeholder for jump out of conditional suite if this
            // branch is selected.
            let jump_out_addr = self.chunk.len();
            jump_out_addrs.push(jump_out_addr);
            self.push(Inst::Placeholder(
                jump_out_addr,
                Box::new(Inst::Jump(0, 0)),
                "Branch jump out not set".to_owned(),
            ));

            // Jump target if branch condition is false.
            let next_addr = self.chunk.len();

            self.chunk[jump_index] = Inst::JumpIfElse(block_addr, next_addr, 0);
        }

        // Default block (if present).
        if let Some(default_block) = default {
            self.visit_block(default_block, scope_kind)?;
        } else {
            self.push(Inst::LoadConst(0));
        }

        // Address of instruction after conditional suite.
        let after_addr = self.chunk.len();

        // Replace jump-out placeholders with actual jumps.
        for addr in jump_out_addrs {
            self.chunk[addr] = Inst::Jump(after_addr, 0);
        }

        Ok(())
    }

    fn visit_loop(
        &mut self,
        expr: ast::Expr,
        block: ast::Block,
        scope_kind: ScopeKind,
    ) -> VisitResult {
        let loop_scope_depth = self.scope_depth;
        let loop_addr = self.chunk.len();
        // Evaluate loop expression on every iteration.
        self.visit_expr(expr)?;
        // Placeholder for jump-out if result is false.
        let jump_out_index = self.chunk.len();
        self.push(Inst::Placeholder(
            jump_out_index,
            Box::new(Inst::JumpIfNot(0, 0)),
            "Jump-out for loop not set".to_owned(),
        ));
        // Run the loop body if result is true.
        self.visit_block(block, scope_kind)?;
        // Jump to top of loop to re-check expression.
        self.push(Inst::Jump(loop_addr, 0));
        // Jump-out address.
        let after_addr = self.chunk.len();
        // Set address of jump-out placeholder.
        self.chunk[jump_out_index] = Inst::JumpIfNot(after_addr, 0);
        // Set address of breaks and continues.
        for addr in loop_addr..after_addr {
            match self.chunk[addr] {
                Inst::BreakPlaceholder(break_addr, depth) => {
                    self.chunk[break_addr] =
                        Inst::Jump(after_addr, depth - loop_scope_depth);
                }
                Inst::ContinuePlaceholder(continue_addr, depth) => {
                    self.chunk[continue_addr] =
                        Inst::Jump(loop_addr, depth - loop_scope_depth);
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn visit_break(&mut self, expr: ast::Expr) -> VisitResult {
        self.visit_expr(expr)?;
        self.chunk.push(Inst::BreakPlaceholder(self.chunk.len(), self.scope_depth));
        Ok(())
    }

    fn visit_continue(&mut self) -> VisitResult {
        self.chunk.push(Inst::ContinuePlaceholder(self.chunk.len(), self.scope_depth));
        Ok(())
    }

    fn visit_func(&mut self, node: ast::Func) -> VisitResult {
        let mut func_visitor = Visitor::new(&mut self.ctx);
        let name = node.name;
        let params = node.params;
        let func_program = ast::Program::new(node.block.statements);
        if name == "$main" && self.scope_tree.in_global_scope() {
            self.has_main = true;
        }
        func_visitor.visit_program(func_program)?;
        let chunk = func_visitor.chunk;
        self.push(Inst::DeclareVar(name.clone()));
        self.add_const(self.ctx.builtins.new_func(name.clone(), params, chunk));
        self.push(Inst::AssignVar(name));
        Ok(())
    }

    fn visit_call(&mut self, node: ast::Call) -> VisitResult {
        let name = node.name;
        let args = node.args;
        let n_args = args.len();
        self.push(Inst::LoadVar(name));
        for arg in args {
            self.visit_expr(arg)?;
        }
        self.push(Inst::ScopeStart);
        self.push(Inst::Call(n_args));
        self.push(Inst::ScopeEnd);
        self.push(Inst::Return);
        Ok(())
    }

    fn visit_unary_op(&mut self, op: UnaryOperator, expr: ast::Expr) -> VisitResult {
        self.visit_expr(expr)?;
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
            Assign => self.visit_assignment(expr_a, expr_b),
            _ => {
                self.visit_expr(expr_a)?;
                self.visit_expr(expr_b)?;
                self.push(Inst::BinaryOp(op));
                Ok(())
            }
        }
    }

    // Utilities -------------------------------------------------------

    fn push(&mut self, inst: Inst) {
        self.chunk.push(inst);
    }

    fn push_const(&mut self, index: usize) {
        self.push(Inst::LoadConst(index));
    }

    fn add_const(&mut self, val: ObjectRef) {
        let index = self.ctx.add_const(val);
        self.push_const(index);
    }

    /// Add nested scope to current scope then make the new scope the
    /// current scope.
    fn enter_scope(&mut self, kind: ScopeKind) {
        self.scope_tree.add(kind);
        self.scope_depth += 1;
    }

    /// Move up to the parent scope of the current scope.
    fn exit_scope(&mut self) {
        self.scope_tree.move_up();
        self.scope_depth -= 1;
    }

    /// Update jump instructions with their target label addresses.
    fn fix_jumps(&mut self) -> VisitResult {
        let chunk = &mut self.chunk;
        let scope_tree = &self.scope_tree;
        let mut not_found: Option<String> = None;
        scope_tree.walk_up(&mut |scope: &Scope, jump_depth: usize| {
            for (name, jump_addr) in scope.jumps().iter() {
                let result = scope.find_label(scope_tree, name, None);
                if let Some((label_addr, label_depth)) = result {
                    let depth = jump_depth - label_depth;
                    chunk[*jump_addr] = Inst::Jump(label_addr, depth);
                } else {
                    not_found = Some(name.clone());
                    return false;
                }
            }
            true
        });
        if let Some(name) = not_found {
            return Err(CompErr::new_label_not_found_in_scope(name));
        }
        Ok(())
    }
}
