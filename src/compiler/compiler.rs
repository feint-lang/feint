use crate::ast;
use crate::types::ObjectRef;
use crate::util::{BinaryOperator, UnaryOperator};
use crate::vm::{Instruction, Instructions, RuntimeContext, VM};

use super::result::{CompilationError, CompilationErrorKind, CompilationResult};
use super::scope::{Scope, ScopeKind, ScopeTree};

// Compiler ------------------------------------------------------------

/// Compile AST to VM instructions.
pub fn compile(vm: &mut VM, program: ast::Program, _debug: bool) -> CompilationResult {
    let mut visitor = Visitor::new(&mut vm.ctx);
    visitor.visit_program(program)?;
    Ok(visitor.instructions)
}

// Visitor -------------------------------------------------------------

type VisitResult = Result<(), CompilationError>;

struct Visitor<'a> {
    ctx: &'a mut RuntimeContext,
    instructions: Instructions,
    scope_tree: ScopeTree,
}

impl<'a> Visitor<'a> {
    fn new(ctx: &'a mut RuntimeContext) -> Self {
        Self { ctx, instructions: Instructions::new(), scope_tree: ScopeTree::new() }
    }

    // Utilities -------------------------------------------------------

    fn err(&self, message: String) -> VisitResult {
        Err(CompilationError::new(CompilationErrorKind::VisitError(message)))
    }

    fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    fn push_const(&mut self, index: usize) {
        self.push(Instruction::LoadConst(index));
    }

    fn add_const(&mut self, val: ObjectRef) {
        let index = self.ctx.constants.add(val);
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
                        0 => Instruction::NoOp,
                        _ => Instruction::ScopeEnd(depth),
                    };
                    instructions[*jump_addr] = Instruction::Jump(label_addr);
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
        self.push(Instruction::Halt(0));
        Ok(())
    }

    fn visit_block(&mut self, node: ast::Block, scope_kind: ScopeKind) -> VisitResult {
        self.push(Instruction::ScopeStart);
        self.enter_scope(scope_kind);
        for statement in node.statements {
            self.visit_statement(statement)?;
        }
        self.push(Instruction::ScopeEnd(1));
        self.exit_scope();
        Ok(())
    }

    fn visit_statement(&mut self, node: ast::Statement) -> VisitResult {
        match node.kind {
            ast::StatementKind::Print => {
                self.push(Instruction::Print);
                self.push(Instruction::Push(0));
            }
            ast::StatementKind::Jump(name) => {
                // Insert placeholder jump instruction to be filled in
                // with corresponding label address once labels have
                // been processed. We also take care to exit nested
                // blocks/scopes before jumping out.
                self.push(Instruction::ScopeEnd(1));
                self.push(Instruction::Jump(0));
                let addr = self.instructions.len() - 1;
                self.scope_tree.add_jump(name.as_str(), addr);
            }
            ast::StatementKind::Label(name) => {
                self.push(Instruction::NoOp);
                let addr = self.instructions.len() - 1;
                if self.scope_tree.add_label(name.as_str(), addr).is_some() {
                    self.err(format!("Duplicate label in scope: {}", name))?;
                }
            }
            ast::StatementKind::Expr(expr) => self.visit_expr(expr)?,
        }
        Ok(())
    }

    fn visit_expr(&mut self, node: ast::Expr) -> VisitResult {
        match node.kind {
            ast::ExprKind::Block(block) => self.visit_block(block, ScopeKind::Block)?,
            ast::ExprKind::UnaryOp(op, b) => self.visit_unary_op(op, *b)?,
            ast::ExprKind::BinaryOp(a, op, b) => self.visit_binary_op(*a, op, *b)?,
            ast::ExprKind::Ident(ident) => self.visit_ident(ident)?,
            ast::ExprKind::Literal(literal) => self.visit_literal(literal)?,
            _ => self.err(format!("Unhandled expression: {:?}", node))?,
        }
        Ok(())
    }

    fn visit_unary_op(&mut self, op: UnaryOperator, expr: ast::Expr) -> VisitResult {
        self.visit_expr(expr)?;
        self.push(Instruction::UnaryOp(op));
        Ok(())
    }

    fn visit_binary_op(
        &mut self,
        expr_a: ast::Expr,
        op: BinaryOperator,
        expr_b: ast::Expr,
    ) -> VisitResult {
        if let BinaryOperator::Assign = op {
            self.visit_assignment(expr_a, expr_b)
        } else {
            self.visit_expr(expr_a)?;
            self.visit_expr(expr_b)?;
            self.push(Instruction::BinaryOp(op));
            Ok(())
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
                    self.push(Instruction::DeclareVar(name.clone()));
                    self.visit_expr(value_expr)?;
                    self.push(Instruction::AssignVar(name));
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
        match node.kind {
            ast::IdentKind::Ident(name) => self.push(Instruction::LoadVar(name)),
            ast::IdentKind::TypeIdent(name) => self.push(Instruction::LoadVar(name)),
        }
        Ok(())
    }

    fn visit_literal(&mut self, node: ast::Literal) -> VisitResult {
        match node.kind {
            ast::LiteralKind::Nil => self.push_const(0),
            ast::LiteralKind::Bool(true) => self.push_const(1),
            ast::LiteralKind::Bool(false) => self.push_const(2),
            ast::LiteralKind::Float(value) => {
                self.add_const(self.ctx.builtins.new_float(value))
            }
            ast::LiteralKind::Int(value) => {
                self.add_const(self.ctx.builtins.new_int(value))
            }
            ast::LiteralKind::String(value) => {
                self.add_const(self.ctx.builtins.new_string(value, false))
            }
            ast::LiteralKind::FormatString(value) => {
                self.add_const(self.ctx.builtins.new_string(value, true))
            }
        }
        Ok(())
    }
}
