use std::collections::HashMap;

use crate::ast;
use crate::ast::LiteralKind::Int;
use crate::types::ObjectRef;
use crate::util::{BinaryOperator, UnaryOperator};
use crate::vm::{Instruction, Instructions, RuntimeContext, VM};

use super::result::{CompilationError, CompilationErrorKind, CompilationResult};
use super::scope::{Scope, ScopeTree};

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

    fn enter_scope(&mut self) {
        // Add child scope to current scope, then make child the current
        // scope.
        self.scope_tree.add();
    }

    fn exit_scope(&mut self) {
        // Move up to the parent scope of the current scope.
        self.scope_tree.move_up();
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

    /// Update jump instructions with their corresponding label
    /// addresses.
    fn fix_jumps(&mut self) -> VisitResult {
        let mut scope_tree = &self.scope_tree;
        let mut updated: HashMap<usize, Instruction> = HashMap::new();

        scope_tree.walk_up(&mut |scope: &Scope, depth: usize| {
            let all_jumps = scope_tree.all_jumps_for_scope(scope.index());
            for jumps in all_jumps.iter() {
                for (name, jump_addr) in jumps.iter() {
                    if updated.contains_key(jump_addr) {
                        // The label for this jump was already found in
                        // a nested scope.
                        continue;
                    }
                    if let Some(label_addr) = scope.labels().get(name) {
                        updated.insert(
                            *jump_addr - 1,
                            match depth {
                                0 => Instruction::NoOp,
                                _ => Instruction::BlockEnd(depth),
                            },
                        );
                        updated.insert(*jump_addr, Instruction::Jump(*label_addr));
                    }
                }
            }
            true
        });

        let mut not_found: Option<(String, usize)> = None;
        scope_tree.walk_up(&mut |scope: &Scope, depth: usize| {
            for (name, jump_addr) in scope.jumps().iter() {
                if !updated.contains_key(jump_addr) {
                    not_found = Some((name.clone(), jump_addr.clone()));
                    return false;
                }
            }
            true
        });

        if let Some(item) = not_found {
            return self.err(format!("Label not found for jump {}", item.0));
        }

        for (addr, inst) in updated {
            self.instructions[addr] = inst;
        }

        Ok(())
    }

    fn visit_block(&mut self, node: ast::Block) -> VisitResult {
        self.push(Instruction::BlockStart);
        self.enter_scope();
        for statement in node.statements {
            self.visit_statement(statement)?;
        }
        self.push(Instruction::BlockEnd(1));
        self.exit_scope();
        Ok(())
    }

    fn visit_statement(&mut self, node: ast::Statement) -> VisitResult {
        match node.kind {
            ast::StatementKind::Print => self.push(Instruction::Print),
            ast::StatementKind::Jump(name) => {
                // Insert placeholder jump instruction to be filled in
                // with corresponding label address once labels have
                // been processed.
                self.push(Instruction::BlockEnd(1));
                self.push(Instruction::Jump(0));
                self.scope_tree.add_jump(name.as_str(), self.instructions.len() - 1);
            }
            ast::StatementKind::Label(name) => {
                self.push(Instruction::NoOp);
                let result = self
                    .scope_tree
                    .add_label(name.as_str(), self.instructions.len() - 1);
                if result.is_some() {
                    self.err(format!("Duplicate label in scope: {}", name))?;
                }
            }
            ast::StatementKind::Expr(expr) => self.visit_expr(expr)?,
        }
        Ok(())
    }

    fn visit_expr(&mut self, node: ast::Expr) -> VisitResult {
        match node.kind {
            ast::ExprKind::Block(block) => self.visit_block(block)?,
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
