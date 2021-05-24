use std::rc::Rc;

use num_traits::cast::ToPrimitive;

use crate::ast;
use crate::types::builtins::Builtins;
use crate::util::BinaryOperator;
use crate::vm::{format_instructions, Instruction, Instructions, ObjectStore};

use super::result::{CompilationError, CompilationErrorKind, CompilationResult};

type VisitResult = Result<(), CompilationError>;

/// Compile AST to VM instructions.
pub fn compile(
    builtins: &Builtins,
    object_store: &ObjectStore,
    program: ast::Program,
    debug: bool,
) -> CompilationResult {
    if debug {
        eprintln!("COMPILING:\n{:?}", program);
    }

    let mut visitor = Visitor::new(builtins, object_store);
    visitor.visit_program(program)?;

    if debug {
        eprintln!("INSTRUCTIONS:\n{}", format_instructions(&visitor.instructions));
    }

    Ok(visitor.instructions)
}

struct Visitor<'a> {
    builtins: &'a Builtins,
    object_store: &'a ObjectStore,
    instructions: Instructions,
}

impl<'a> Visitor<'a> {
    fn new(builtins: &'a Builtins, object_store: &'a ObjectStore) -> Self {
        Self { builtins, object_store, instructions: Instructions::new() }
    }

    fn err(&self, message: String) -> VisitResult {
        Err(CompilationError::new(CompilationErrorKind::VisitationError(message)))
    }

    fn visit_program(&mut self, node: ast::Program) -> VisitResult {
        self.instructions.push(Instruction::StoreConst(self.builtins.nil_obj.clone()));
        self.instructions.push(Instruction::StoreConst(self.builtins.true_obj.clone()));
        self.instructions
            .push(Instruction::StoreConst(self.builtins.false_obj.clone()));
        for statement in node.statements {
            self.visit_statement(statement)?;
        }
        self.instructions.push(Instruction::Print);
        self.instructions.push(Instruction::Halt(0));
        Ok(())
    }

    fn visit_statement(&mut self, node: ast::Statement) -> VisitResult {
        match node.kind {
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
        self.instructions.push(Instruction::BinaryOp(op));
        Ok(())
    }

    fn visit_literal(&mut self, node: ast::Literal) -> VisitResult {
        match node.kind {
            // ast::LiteralKind::Float(value) => self.instructions.push(value),
            ast::LiteralKind::Int(value) => {
                let value = Rc::new(self.builtins.new_int(value));
                // let index = self.instructions.push(Instruction::StoreConst(value));
            }
            _ => self.err(format!("Unhandled literal: {:?}", node))?,
        }
        Ok(())
    }
}
