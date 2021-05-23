use std::rc::Rc;

use num_traits::cast::ToPrimitive;

use crate::ast;
use crate::types::builtins;
use crate::util::BinaryOperator;
use crate::vm::{format_instructions, Instruction, Instructions};

use super::result::CompilationResult;

/// Compile AST to VM instructions.
pub fn compile(program: ast::Program, debug: bool) -> CompilationResult {
    if debug {
        eprintln!("COMPILING:\n{:?}", program);
    }

    let instructions = Visitor::visit(program);

    if debug {
        eprintln!("INSTRUCTIONS:\n{}", format_instructions(&instructions));
    }

    Ok(instructions)
}

struct Visitor {
    instructions: Instructions,
}

impl Visitor {
    fn new(instructions: Instructions) -> Self {
        Self { instructions }
    }

    fn visit(program: ast::Program) -> Instructions {
        let mut visitor = Visitor::default();
        visitor.visit_program(program);
        visitor.instructions
    }

    fn visit_program(&mut self, node: ast::Program) {
        for statement in node.statements {
            self.visit_statement(statement);
        }
        self.instructions.push(Instruction::Print);
        self.instructions.push(Instruction::Halt(0));
    }

    fn visit_statement(&mut self, node: ast::Statement) {
        match node.kind {
            ast::StatementKind::Expr(expr) => self.visit_expr(*expr),
            _ => panic!("Unhandled statement: {:?}", node),
        }
    }

    fn visit_expr(&mut self, node: ast::Expr) {
        match node.kind {
            ast::ExprKind::BinaryOperation(a, op, b) => {
                self.visit_binary_operation(*a, op, *b)
            }
            ast::ExprKind::Literal(literal) => self.visit_literal(*literal),
            _ => panic!("Unhandled expr: {:?}", node),
        }
    }

    fn visit_binary_operation(
        &mut self,
        expr_a: ast::Expr,
        op: BinaryOperator,
        expr_b: ast::Expr,
    ) {
        self.visit_expr(expr_a);
        self.visit_expr(expr_b);
        self.instructions.push(Instruction::BinaryOp(op));
    }

    fn visit_literal(&mut self, node: ast::Literal) {
        match node.kind {
            // ast::LiteralKind::Float(value) => self.instructions.push(value),
            ast::LiteralKind::Int(value) => {
                let value = Rc::new(builtins::Int::from(value));
                self.instructions.push(Instruction::StoreConst(value));
            }
            _ => panic!("Unhandled literal: {:?}", node),
        }
    }
}

impl Default for Visitor {
    fn default() -> Self {
        let instructions = Instructions::new();
        Visitor::new(instructions)
    }
}
