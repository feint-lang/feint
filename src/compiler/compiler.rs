use std::rc::Rc;

use num_traits::cast::ToPrimitive;

use crate::ast;
use crate::types::{builtins::Builtins, ObjectRef};
use crate::util::BinaryOperator;
use crate::vm::{format_instructions, Instruction, Instructions, ObjectStore};

use super::result::{CompilationError, CompilationErrorKind, CompilationResult};

type VisitResult = Result<(), CompilationError>;

/// Compile AST to VM instructions.
pub fn compile(
    builtins: &Builtins,
    object_store: &mut ObjectStore,
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
    object_store: &'a mut ObjectStore,
    instructions: Instructions,
}

impl<'a> Visitor<'a> {
    fn new(builtins: &'a Builtins, object_store: &'a mut ObjectStore) -> Self {
        Self { builtins, object_store, instructions: Instructions::new() }
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
                if let Some(expr) = maybe_expr {
                    self.visit_expr(*expr)?;
                }
                self.push(Instruction::Print);
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
        let index = self.object_store.add(val);
        self.push_const(index);
    }

    fn visit_literal(&mut self, node: ast::Literal) -> VisitResult {
        match node.kind {
            ast::LiteralKind::Nil => self.push_const(0),
            ast::LiteralKind::Bool(true) => self.push_const(1),
            ast::LiteralKind::Bool(false) => self.push_const(2),
            ast::LiteralKind::Float(value) => {
                self.add_const(self.builtins.new_float(value))
            }
            ast::LiteralKind::Int(value) => {
                self.add_const(self.builtins.new_int(value))
            }
            _ => return self.err(format!("Unhandled literal: {:?}", node)),
        }
        Ok(())
    }
}