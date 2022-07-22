use crate::ast;
use crate::types::ObjectRef;
use crate::util::{BinaryOperator, UnaryOperator};
use crate::vm::{Chunk, Inst, RuntimeContext, VM};

use super::result::{CompilationErr, CompilationErrKind, CompilationResult};
use super::scope::{Scope, ScopeKind, ScopeTree};

// Compiler ------------------------------------------------------------

/// Compile AST to VM instructions.
pub fn compile(vm: &mut VM, program: ast::Program) -> CompilationResult {
    let mut visitor = Visitor::new(&mut vm.ctx);
    visitor.visit_program(program)?;
    Ok(visitor.instructions)
}

// Visitor -------------------------------------------------------------

type VisitResult = Result<(), CompilationErr>;

struct Visitor<'a> {
    ctx: &'a mut RuntimeContext,
    instructions: Chunk,
    scope_tree: ScopeTree,
}

impl<'a> Visitor<'a> {
    fn new(ctx: &'a mut RuntimeContext) -> Self {
        Self { ctx, instructions: Chunk::new(), scope_tree: ScopeTree::new() }
    }

    // Utilities -------------------------------------------------------

    fn err(&self, message: String) -> VisitResult {
        Err(CompilationErr::new(CompilationErrKind::VisitErr(message)))
    }

    fn push(&mut self, inst: Inst) {
        self.instructions.push(inst);
    }

    fn push_const(&mut self, index: usize) {
        self.push(Inst::LoadConst(index));
    }

    fn add_const(&mut self, val: ObjectRef) {
        let index = self.ctx.add_obj(val);
        self.push_const(index);
    }

    /// Add nested scope to current scope then make the new scope the
    /// current scope.
    fn enter_scope(&mut self, kind: ScopeKind) {
        self.scope_tree.add(kind);
    }

    /// Move up to the parent scope of the current scope.
    fn exit_scope(&mut self) {
        self.scope_tree.move_up();
    }

    /// Update jump instructions with their target label addresses.
    fn fix_jumps(&mut self) -> VisitResult {
        let instructions = &mut self.instructions;
        let scope_tree = &self.scope_tree;
        let mut not_found: Option<String> = None;
        scope_tree.walk_up(&mut |scope: &Scope, jump_depth: usize| {
            for (name, jump_addr) in scope.jumps().iter() {
                let result = scope.find_label(scope_tree, name, None);
                if let Some((label_addr, label_depth)) = result {
                    let depth = jump_depth - label_depth;
                    instructions[*jump_addr - 1] = match depth {
                        0 => Inst::NoOp,
                        _ => Inst::ScopeEnd(depth),
                    };
                    instructions[*jump_addr] = Inst::Jump(label_addr);
                } else {
                    not_found = Some(name.clone());
                    return false;
                }
            }
            true
        });
        if let Some(name) = not_found {
            return self.err(format!(
                "Label not found for jump {} (jump target must be *after* jump)",
                name
            ));
        }
        Ok(())
    }

    // Visitors --------------------------------------------------------

    fn visit_program(&mut self, node: ast::Program) -> VisitResult {
        for statement in node.statements {
            self.visit_statement(statement)?;
        }
        assert_eq!(self.scope_tree.pointer(), 0);
        self.fix_jumps()?;
        self.push(Inst::Halt(0));
        Ok(())
    }

    fn visit_block(&mut self, node: ast::Block, scope_kind: ScopeKind) -> VisitResult {
        self.push(Inst::ScopeStart);
        self.enter_scope(scope_kind);
        for statement in node.statements {
            self.visit_statement(statement)?;
        }
        self.push(Inst::ScopeEnd(1));
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
            let jump_index = self.instructions.len();
            self.push(Inst::Placeholder(
                jump_index,
                Box::new(Inst::JumpIfElse(0, 0)),
                "Branch condition jump not set".to_owned(),
            ));

            // Start of branch block (jump target if branch condition is
            // true).
            let block_addr = jump_index + 1;
            self.visit_block(block, scope_kind)?;

            // Placeholder for jump out of conditional suite if this
            // branch is selected.
            let jump_out_addr = self.instructions.len();
            jump_out_addrs.push(jump_out_addr);
            self.push(Inst::Placeholder(
                jump_out_addr,
                Box::new(Inst::Jump(0)),
                "Branch jump out not set".to_owned(),
            ));

            // Jump target if branch condition is false.
            let next_addr = self.instructions.len();

            self.instructions[jump_index] = Inst::JumpIfElse(block_addr, next_addr);
        }

        // Default block (if present).
        if let Some(default_block) = default {
            self.visit_block(default_block, scope_kind)?;
        }

        // Address of instruction after conditional suite.
        let after_addr = self.instructions.len();

        // Replace jump-out placeholders with actual jumps.
        for addr in jump_out_addrs {
            self.instructions[addr] = Inst::Jump(after_addr);
        }

        Ok(())
    }

    fn visit_loop(
        &mut self,
        expr: ast::Expr,
        block: ast::Block,
        scope_kind: ScopeKind,
    ) -> VisitResult {
        let loop_addr = self.instructions.len();
        // Evaluate loop expression on every iteration.
        self.visit_expr(expr)?;
        // Placeholder for jump-out if result is false.
        let jump_out_index = self.instructions.len();
        self.push(Inst::Placeholder(
            jump_out_index,
            Box::new(Inst::JumpIfNot(0)),
            "Jump-out for loop not set".to_owned(),
        ));
        // Run the loop body if result is true.
        self.visit_block(block, scope_kind)?;
        // Jump to top of loop to re-check expression.
        self.push(Inst::Jump(loop_addr));
        // Jump-out address.
        let after_addr = self.instructions.len();
        // Set address of jump-out placeholder.
        self.instructions[jump_out_index] = Inst::JumpIfNot(after_addr);
        // Set address of breaks and continues.
        for addr in loop_addr..after_addr {
            match self.instructions[addr] {
                Inst::BreakPlaceholder(break_addr) => {
                    self.instructions[break_addr] = Inst::Jump(after_addr)
                }
                Inst::ContinuePlaceholder(continue_addr) => {
                    self.instructions[continue_addr] = Inst::Jump(loop_addr)
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn visit_break(&mut self, expr: ast::Expr) -> VisitResult {
        self.visit_expr(expr)?;
        self.instructions.push(Inst::BreakPlaceholder(self.instructions.len()));
        Ok(())
    }

    fn visit_continue(&mut self) -> VisitResult {
        self.instructions.push(Inst::ContinuePlaceholder(self.instructions.len()));
        Ok(())
    }

    fn visit_func(&mut self, node: ast::Func) -> VisitResult {
        eprintln!("IMPLEMENT visit_func()!!!");
        eprintln!("{}({})", node.name, node.params.join(", "));
        Ok(())
    }

    fn visit_call(&mut self, node: ast::Call) -> VisitResult {
        eprintln!("IMPLEMENT visit_call()!!!");
        eprintln!("{}()", node.name);
        Ok(())
    }

    fn visit_statement(&mut self, node: ast::Statement) -> VisitResult {
        type Kind = ast::StatementKind;
        match node.kind {
            Kind::Print(items) => {
                let num_items = items.len();
                for item in items {
                    self.visit_expr(item)?;
                }
                self.push(Inst::Print(num_items));
                self.push(Inst::Push(0));
            }
            Kind::Jump(name) => {
                // Insert placeholder jump instruction to be filled in
                // with corresponding label address once labels have
                // been processed. We also take care to exit nested
                // blocks/scopes before jumping out.
                self.push(Inst::Placeholder(
                    0,
                    Box::new(Inst::ScopeEnd(0)),
                    "Scope not exited for jump".to_owned(),
                ));
                let jump_addr = self.instructions.len();
                self.push(Inst::Placeholder(
                    0,
                    Box::new(Inst::Jump(0)),
                    "Jump address not set to label address".to_owned(),
                ));
                self.scope_tree.add_jump(name.as_str(), jump_addr);
            }
            Kind::Label(name) => {
                self.push(Inst::NoOp);
                let addr = self.instructions.len() - 1;
                if self.scope_tree.add_label(name.as_str(), addr).is_some() {
                    self.err(format!("Duplicate label in scope: {}", name))?;
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
            Kind::Block(block) => self.visit_block(block, ScopeKind::Block)?,
            Kind::Conditional(branches, default) => {
                self.visit_conditional(branches, default, ScopeKind::Block)?
            }
            Kind::Loop(expr, block) => {
                self.visit_loop(*expr, block, ScopeKind::Block)?
            }
            Kind::Break(expr) => self.visit_break(*expr)?,
            Kind::Func(func) => self.visit_func(func)?,
            Kind::UnaryOp(op, b) => self.visit_unary_op(op, *b)?,
            Kind::BinaryOp(a, op, b) => self.visit_binary_op(*a, op, *b)?,
            Kind::Ident(ident) => self.visit_ident(ident)?,
            Kind::Literal(literal) => self.visit_literal(literal)?,
            Kind::FormatString(items) => self.visit_format_string(items)?,
            Kind::Tuple(items) => self.visit_tuple(items)?,
            _ => self.err(format!("Unhandled expression:\n{:?}", node))?,
        }
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
            Assign => self.visit_assignment(expr_a, expr_b),
            _ => {
                self.visit_expr(expr_a)?;
                self.visit_expr(expr_b)?;
                self.push(Inst::BinaryOp(op));
                Ok(())
            }
        }
    }

    fn visit_assignment(
        &mut self,
        name_expr: ast::Expr,
        value_expr: ast::Expr,
    ) -> VisitResult {
        match name_expr.kind {
            ast::ExprKind::Ident(ident) => match ident.kind {
                ast::IdentKind::Ident(name) => {
                    // NOTE: Currently, declaration and assignment are
                    //       the same thing, so declaration doesn't
                    //       do anything particularly useful ATM.
                    self.push(Inst::DeclareVar(name.clone()));
                    self.visit_expr(value_expr)?;
                    self.push(Inst::AssignVar(name));
                }
                _ => return self.err("Expected identifier".to_owned()),
            },
            _ => return self.err("Expected identifier".to_owned()),
        }
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

    fn visit_literal(&mut self, node: ast::Literal) -> VisitResult {
        type Kind = ast::LiteralKind;
        match node.kind {
            Kind::Nil => self.push_const(0),
            Kind::Bool(true) => self.push_const(1),
            Kind::Bool(false) => self.push_const(2),
            Kind::Float(value) => self.add_const(self.ctx.builtins.new_float(value)),
            Kind::Int(value) => self.add_const(self.ctx.builtins.new_int(value)),
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

    fn visit_tuple(&mut self, items: Vec<ast::Expr>) -> VisitResult {
        let num_items = items.len();
        for item in items {
            self.visit_expr(item)?;
        }
        self.push(Inst::MakeTuple(num_items));
        Ok(())
    }
}
