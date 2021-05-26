use std::rc::Rc;

use num_traits::cast::ToPrimitive;

use crate::ast;
use crate::types::ObjectRef;
use crate::util::BinaryOperator;
use crate::vm::{
    format_instructions, Instruction, Instructions, ObjectStore, RuntimeContext, VM,
};

use super::result::{CompilationError, CompilationErrorKind, CompilationResult};
use std::borrow::BorrowMut;

// Compiler ------------------------------------------------------------

/// Compile AST to VM instructions.
pub fn compile(vm: &mut VM, program: ast::Program, debug: bool) -> CompilationResult {
    if debug {
        eprintln!("COMPILING:\n{:?}", program);
    }

    let mut visitor = Visitor::new(&mut vm.ctx);
    visitor.visit_program(program)?;

    if debug {
        eprintln!("INSTRUCTIONS:\n{}", format_instructions(&visitor.instructions));
    }

    Ok(visitor.instructions)
}

// Visitor -------------------------------------------------------------

type VisitResult = Result<(), CompilationError>;

struct Visitor<'a> {
    ctx: &'a mut RuntimeContext,
    instructions: Instructions,
}

impl<'a> Visitor<'a> {
    fn new(ctx: &'a mut RuntimeContext) -> Self {
        Self { ctx, instructions: Instructions::new() }
    }

    fn err(&self, message: String) -> VisitResult {
        Err(CompilationError::new(CompilationErrorKind::VisitationError(message)))
    }

    /// Push instruction
    fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    fn visit_program(&mut self, node: ast::Program) -> VisitResult {
        for statement in node.statements {
            self.visit_statement(statement)?;
        }
        self.push(Instruction::Halt(0));
        Ok(())
    }

    fn visit_statement(&mut self, node: ast::Statement) -> VisitResult {
        match node.kind {
            ast::StatementKind::Print(maybe_expr) => {
                match maybe_expr {
                    Some(expr) => self.visit_expr(*expr)?,
                    None => self.push_const(0),
                }
                self.push(Instruction::Print);
                // XXX: This is sort of like the return value of print
                self.push_const(0);
            }
            ast::StatementKind::Expr(expr) => self.visit_expr(*expr)?,
            _ => self.err(format!("Unhandled statement: {:?}", node))?,
        }
        Ok(())
    }

    fn visit_expr(&mut self, node: ast::Expr) -> VisitResult {
        match node.kind {
            ast::ExprKind::BinaryOperation(a, op, b) => {
                self.visit_binary_operation(*a, op, *b)?
            }
            ast::ExprKind::Literal(literal) => self.visit_literal(*literal)?,
            _ => self.err(format!("Unhandled expression: {:?}", node))?,
        }
        Ok(())
    }

    fn visit_binary_operation(
        &mut self,
        expr_a: ast::Expr,
        op: BinaryOperator,
        expr_b: ast::Expr,
    ) -> VisitResult {
        self.visit_expr(expr_a)?;
        self.visit_expr(expr_b)?;
        self.push(Instruction::BinaryOp(op));
        Ok(())
    }

    fn push_const(&mut self, index: usize) {
        self.push(Instruction::LoadConst(index));
    }

    fn add_const(&mut self, val: ObjectRef) {
        let index = self.ctx.add_object(val);
        self.push_const(index);
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
            _ => return self.err(format!("Unhandled literal: {:?}", node)),
        }
        Ok(())
    }
}
